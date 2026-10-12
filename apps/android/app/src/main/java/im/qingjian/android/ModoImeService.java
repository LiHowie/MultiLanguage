// Android 输入法生命周期、InputConnection 和触摸事件桥接。
package im.modolanguage.android;

import android.content.Intent;
import android.inputmethodservice.InputMethodService;
import android.os.Handler;
import android.os.Looper;
import android.text.InputType;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import org.json.JSONObject;
import java.io.File;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class ModoImeService extends InputMethodService implements KeyboardView.Listener {
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private final Handler main = new Handler(Looper.getMainLooper());
    private KeyboardView keyboard;
    private long handle;
    private String appliedSettings = "";
    private int generation;
    private boolean english;
    private boolean symbols;
    private boolean shift;
    private boolean password;
    private boolean ready;
    private JSONObject frame = new JSONObject();
    private VoiceTeacher teacher;
    private JSONObject speech;
    private boolean t9;
    private int speechGeneration;
    private final Runnable poll = new Runnable() {
        @Override public void run() {
            if (isInputViewShown() && ready && !frame.optString("input", "").isEmpty()) command("poll", "");
            if (isInputViewShown()) main.postDelayed(this, 160);
        }
    };

    @Override public View onCreateInputView() {
        if(teacher!=null)teacher.close();
        keyboard = new KeyboardView(this, this);
        keyboard.layout(t9);
        teacher=new VoiceTeacher(this,new VoiceTeacher.Listener(){
            public void result(JSONObject value){if(speechGeneration!=generation||password||frame.optBoolean("private"))return;speech=value;if(value.optString("mode").equals("native")){InputConnection c=getCurrentInputConnection();if(c!=null)c.commitText(value.optString("output"),1);}keyboard.speech(value);keyboard.message("完成 · AI 辅助");}
            public void message(String value){if(keyboard!=null)keyboard.message(value);}
        });return keyboard;
    }
    @Override public boolean onEvaluateFullscreenMode() { return false; }
    @Override public boolean onEvaluateInputViewShown() { return true; }
    @Override public void onStartInput(EditorInfo info, boolean restarting) {
        super.onStartInput(info, restarting);
        int token = ++generation;
        if(teacher!=null)teacher.stop(true);
        speech=null;if(keyboard!=null)keyboard.speech(null);
        t9=new SettingsStore(this).layout().equals("t9");
        if(keyboard!=null)keyboard.layout(t9);
        ready = false; frame = new JSONObject();
        password = isPassword(info.inputType);
        int type = info.inputType & InputType.TYPE_MASK_CLASS;
        english = password || type != InputType.TYPE_CLASS_TEXT ||
            (info.inputType & InputType.TYPE_MASK_VARIATION) == InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS ||
            (info.inputType & InputType.TYPE_MASK_VARIATION) == InputType.TYPE_TEXT_VARIATION_URI;
        symbols = type == InputType.TYPE_CLASS_NUMBER || type == InputType.TYPE_CLASS_PHONE || type == InputType.TYPE_CLASS_DATETIME;
        shift = false;
        boolean privateInput = password || (info.imeOptions & EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING) != 0;
        worker.execute(() -> {
            try {
                String config = new SettingsStore(this).json();
                if (handle == 0) {
                    File data = ProductData.prepare(this);
                    handle = NativeEngine.create(data.getAbsolutePath(), new File(getFilesDir(), "user").getAbsolutePath(), config);
                    appliedSettings = config;
                } else if (!config.equals(appliedSettings)) {
                    NativeEngine.configure(handle, config); appliedSettings = config;
                }
                JSONObject updated = new JSONObject(NativeEngine.command(handle, "reset", privateInput ? "private" : ""));
                updated = new JSONObject(NativeEngine.command(handle, "mode", english ? "en" : "zh"));
                final JSONObject initialFrame = updated;
                main.post(() -> { if (token == generation) { ready = true; apply(initialFrame); } });
            } catch (Exception | LinkageError error) { fail(token, error); }
        });
    }
    @Override public void onStartInputView(EditorInfo info, boolean restarting) {
        super.onStartInputView(info, restarting);
        t9 = new SettingsStore(this).layout().equals("t9");
        if (keyboard != null) { keyboard.layout(t9); keyboard.mode(english, symbols, shift); keyboard.frame(frame); }
        main.removeCallbacks(poll); main.postDelayed(poll, 160);
    }
    @Override public void onFinishInput() {
        if(teacher!=null)teacher.stop(true);
        ++generation; ready = false; main.removeCallbacks(poll);
        worker.execute(() -> { if (handle != 0) NativeEngine.command(handle, "reset", "private"); });
        super.onFinishInput();
    }
    @Override public void onFinishInputView(boolean finishing) {
        if(teacher!=null)teacher.stop(true);
        main.removeCallbacks(poll); super.onFinishInputView(finishing);
    }
    private void command(String action, String value) {
        int token = generation;
        worker.execute(() -> {
            try {
                if (handle == 0) return;
                JSONObject updated = new JSONObject(NativeEngine.command(handle, action, value));
                main.post(() -> {
                    if (token == generation && (!action.equals("poll") || !frame.toString().equals(updated.toString()))) apply(updated);
                });
            } catch (Exception | LinkageError error) { fail(token, error); }
        });
    }
    private void apply(JSONObject updated) {
        frame = updated;
        InputConnection connection = getCurrentInputConnection();
        if (connection != null) {
            connection.beginBatchEdit();
            if (!updated.isNull("commit")) connection.commitText(updated.optString("commit"), 1);
            if (updated.optBoolean("delete")) connection.deleteSurroundingTextInCodePoints(1, 0);
            String composing = updated.optString("input", "");
            if (!composing.isEmpty() && !password) connection.setComposingText(composing, 1);
            else connection.finishComposingText();
            connection.endBatchEdit();
        }
        if (keyboard != null) { keyboard.mode(english, symbols, shift); keyboard.frame(updated); }
    }
    private void fail(int token, Throwable error) {
        main.post(() -> { if (token == generation && keyboard != null) keyboard.message("输入法初始化失败，请重新打开键盘：" + error.getMessage()); });
    }
    @Override public void onUpdateSelection(int oldStart, int oldEnd, int newStart, int newEnd, int candidatesStart, int candidatesEnd) {
        super.onUpdateSelection(oldStart, oldEnd, newStart, newEnd, candidatesStart, candidatesEnd);
        if (!frame.optString("input", "").isEmpty() && (newStart != candidatesEnd || newEnd != candidatesEnd)) {
            InputConnection connection = getCurrentInputConnection();
            if (connection != null) connection.finishComposingText();
            command("clear", "");
        }
    }
    @Override public void candidate(int index, boolean translation) { command(translation ? "translate" : "select", Integer.toString(index)); }
    @Override public void key(String value) {
        InputConnection connection = getCurrentInputConnection(); if (connection == null) return;
        switch (value) {
            case "settings":
                startActivity(new Intent(this, ModoSettingsActivity.class).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)); return;
            case "speech-output":
                if(speech!=null&&!password&&!frame.optBoolean("private"))connection.commitText(speech.optString("output",speech.optString("correctedText")),1);return;
            case "26键":
                command("clear","");t9=false;keyboard.layout(false);
                SettingsStore s=new SettingsStore(this);s.local(s.language(),s.style(),"qwerty",s.voiceMode(),s.consent(),s.enabled());return;
            case "translate": candidate(0, true); return;
            case "🌐": ((android.view.inputmethod.InputMethodManager) getSystemService(INPUT_METHOD_SERVICE)).showInputMethodPicker(); return;
            case "符号":
            case "123": symbols = true; break;
            case "ABC": symbols = false; break;
            case "⇧": shift = !shift; break;
            case "中英":
                if (!frame.optString("input", "").isEmpty()) command("raw", "");
                english = !english; break;
            case "⌫":
                if (password) connection.deleteSurroundingTextInCodePoints(1, 0);
                else command("backspace", ""); return;
            case "空格":
                if (password) connection.commitText(" ", 1); else command("space", ""); return;
            case "↵":
                if (!frame.optString("input", "").isEmpty()) command("space", "");
                else if (!sendDefaultEditorAction(true)) connection.commitText("\n", 1); return;
            default:
                if(value.length()>1&&value.charAt(0)>='2'&&value.charAt(0)<='9'&&value.contains("·")){command("push9",value.substring(0,1));return;}
                if (value.length() == 1 && value.charAt(0) >= 'a' && value.charAt(0) <= 'z') {
                    if (password) connection.commitText(shift ? value.toUpperCase(java.util.Locale.ROOT) : value, 1);
                    else command("push", shift ? value.toUpperCase(java.util.Locale.ROOT) : value);
                } else if (password) connection.commitText(value, 1);
                else command("text", value);
                return;
        }
        if(value.equals("中英"))command("mode",english?"en":"zh");
        if (keyboard != null) keyboard.mode(english, symbols, shift);
    }
    @Override public void voice(String action){
        if(teacher==null)return;
        if(action.equals("cancel")){teacher.stop(true);return;}
        if(action.equals("stop")){teacher.stop(false);return;}
        if(password||frame.optBoolean("private")||!ready){keyboard.message("当前输入框不支持云端语音");return;}
        if(!frame.optString("input").isEmpty()){keyboard.message("请先完成当前选词，再按住空格说话");return;}
        speechGeneration=generation;
        if(action.equals("follow")&&speech!=null)keyboard.message("跟读："+speech.optString("correctedText"));
        teacher.start();
    }
    @Override public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (event.isCtrlPressed() || event.isAltPressed()) return super.onKeyDown(keyCode, event);
        if (keyCode == KeyEvent.KEYCODE_DEL) { key("⌫"); return true; }
        if (keyCode == KeyEvent.KEYCODE_SPACE) { key("空格"); return true; }
        if (keyCode == KeyEvent.KEYCODE_ENTER) { key("↵"); return true; }
        int character = event.getUnicodeChar();
        if (character >= 'a' && character <= 'z') { key(Character.toString((char) character)); return true; }
        return super.onKeyDown(keyCode, event);
    }
    static boolean isPassword(int inputType) {
        int type = inputType & InputType.TYPE_MASK_CLASS;
        int variation = inputType & InputType.TYPE_MASK_VARIATION;
        return (type == InputType.TYPE_CLASS_NUMBER && variation == InputType.TYPE_NUMBER_VARIATION_PASSWORD) ||
            (type == InputType.TYPE_CLASS_TEXT && (variation == InputType.TYPE_TEXT_VARIATION_PASSWORD ||
                variation == InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD || variation == InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD));
    }
    @Override public void onDestroy() {
        if(teacher!=null)teacher.close();
        ++generation; main.removeCallbacksAndMessages(null);
        worker.execute(() -> { if (handle != 0) { NativeEngine.destroy(handle); handle = 0; } });
        worker.shutdown(); super.onDestroy();
    }
}
