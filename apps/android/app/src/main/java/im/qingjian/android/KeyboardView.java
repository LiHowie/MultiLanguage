// 键盘显示面用 Canvas 自绘；触摸仅回传键或候选下标，不处理组句与翻译。
package im.modolanguage.android;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.graphics.RectF;
import android.text.Layout;
import android.text.StaticLayout;
import android.text.TextPaint;
import android.view.MotionEvent;
import android.view.View;
import android.view.WindowInsets;
import org.json.JSONArray;
import org.json.JSONObject;
import java.util.ArrayList;
import java.util.List;

final class KeyboardView extends View {
    interface Listener { void key(String value); void candidate(int index, boolean translation); void voice(String action); }
    private final Listener listener;
    private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final TextPaint text = new TextPaint(Paint.ANTI_ALIAS_FLAG);
    private final List<RectF> keyBounds = new ArrayList<>();
    private final List<String> keyValues = new ArrayList<>();
    private final List<RectF> candidateBounds = new ArrayList<>();
    private final List<Integer> candidateIndices = new ArrayList<>();
    private JSONObject frame = new JSONObject();
    private boolean english;
    private boolean symbols;
    private boolean shift;
    private String message = "正在准备词库…";
    private final float density;
    private float translationTop;
    private float candidateTop;
    private boolean hasTranslation;
    private int candidateOffset;
    private int visibleCandidates;
    private float touchX;
    private float touchY;
    private int bottomInset;
    private boolean t9;
    private boolean voiceHeld;
    private String heldKey = "";
    private JSONObject speech;
    private final android.os.Handler holdHandler = new android.os.Handler(android.os.Looper.getMainLooper());
    private final Runnable longSpace;

    KeyboardView(Context context, Listener listener) {
        super(context); this.listener = listener; density = getResources().getDisplayMetrics().density;
        longSpace = () -> { voiceHeld=true; this.listener.voice("start"); };
        setBackgroundColor(0xffedf2ef); setContentDescription("魔多键盘");
    }
    void frame(JSONObject value) {
        if (!frame.optString("input").equals(value.optString("input"))) candidateOffset = 0;
        frame = value; message = "";
        JSONObject first = rows().optJSONObject(0);
        setContentDescription("魔多键盘。" + (first == null ? "" : first.optString("text")) + "。" + translation());
        requestLayout(); invalidate();
    }
    void message(String value) { message = value; invalidate(); }
    void layout(boolean value) { t9=value; requestLayout(); invalidate(); }
    void speech(JSONObject value) { speech=value; requestLayout(); invalidate(); }
    void mode(boolean english, boolean symbols, boolean shift) {
        this.english = english; this.symbols = symbols; this.shift = shift;
        requestLayout(); invalidate();
    }
    private float dp(float value) { return value * density; }
    private void label(Canvas canvas, String value, float x, float y, float size, int color) {
        paint.setColor(color); paint.setTextSize(dp(size)); paint.setTextAlign(Paint.Align.LEFT);
        canvas.drawText(value, x, y, paint);
    }
    private JSONArray rows() { return frame.optJSONArray("candidates") == null ? new JSONArray() : frame.optJSONArray("candidates"); }
    private String translation() { JSONObject first = rows().optJSONObject(0); return first == null ? "" : first.optString("translation", ""); }
    private StaticLayout paragraph(String value, int width) {
        text.setTextSize(dp(15)); text.setColor(0xff146b54);
        return StaticLayout.Builder.obtain(value, 0, value.length(), text, Math.max(1, width))
            .setAlignment(Layout.Alignment.ALIGN_NORMAL).setMaxLines(3).setEllipsize(android.text.TextUtils.TruncateAt.END)
            .setIncludePad(false).setLineSpacing(dp(3), 1).build();
    }
    @Override public WindowInsets onApplyWindowInsets(WindowInsets insets) {
        int bottom = android.os.Build.VERSION.SDK_INT >= 30 ? insets.getInsets(WindowInsets.Type.systemBars()).bottom :
            insets.getSystemWindowInsetBottom();
        if (bottomInset != bottom) { bottomInset = bottom; requestLayout(); }
        return super.onApplyWindowInsets(insets);
    }
    @Override protected void onMeasure(int widthSpec, int heightSpec) {
        int width = MeasureSpec.getSize(widthSpec);
        String translated = translation();
        hasTranslation = !translated.isEmpty() && !translated.equals("null");
        translationTop = dp(38);
        float translationHeight = hasTranslation ? Math.max(dp(52), paragraph(translated, width - (int) dp(114)).getHeight() + dp(18)) : dp(32);
        candidateTop = translationTop + translationHeight + (speech==null?0:dp(115));
        setMeasuredDimension(width, Math.round(candidateTop + dp(45 + 4 * 49 + 8)) + bottomInset);
    }
    @Override protected void onDraw(Canvas canvas) {
        super.onDraw(canvas); keyBounds.clear(); keyValues.clear(); candidateBounds.clear(); candidateIndices.clear();
        float width = getWidth();
        String input = frame.optString("input", "");
        label(canvas, input.isEmpty() ? "魔多 · " + (english ? "EN" : "拼音") : input, dp(12), dp(26), 15, 0xff143d30);
        label(canvas, "设置", width - dp(47), dp(26), 13, 0xff146b54);
        hit(new RectF(width - dp(61), 0, width, dp(38)), "settings");
        if (hasTranslation) {
            paint.setColor(0xffe0eee7); canvas.drawRoundRect(new RectF(dp(6), translationTop, width - dp(6), candidateTop - dp(3)), dp(10), dp(10), paint);
            canvas.save(); canvas.translate(dp(13), translationTop + dp(9));
            paragraph(translation(), (int) (width - dp(114))).draw(canvas); canvas.restore();
            label(canvas, "输出译文", width - dp(86), translationTop + dp(31), 13, 0xff146b54);
            hit(new RectF(width - dp(97), translationTop, width, candidateTop), "translate");
        } else {
            String hint = !message.isEmpty() ? message : frame.optString("error", "");
            if (hint.isEmpty() || hint.equals("null")) hint = frame.optBoolean("private") ? "私密输入 · 翻译已暂停" :
                (frame.optBoolean("translation_enabled") ? "停键后自动翻译整句" : "整句翻译可在设置中开启");
            label(canvas, hint, dp(12), translationTop + dp(21), 12, 0xff647b70);
        }
        if(speech!=null) {
            float top=candidateTop-dp(115);
            String value=speech.optString("mode").equals("native")?speech.optString("output"):
                speech.optString("motherMeaning")+"\n"+speech.optString("correctedText")+"\n"+speech.optString("explanation");
            canvas.save();canvas.translate(dp(12),top+dp(5));paragraph(value,(int)(width-dp(24))).draw(canvas);canvas.restore();
            label(canvas,"输出",dp(15),candidateTop-dp(12),13,0xff146b54);hit(new RectF(0,candidateTop-dp(38),dp(75),candidateTop),"speech-output");
            label(canvas,"按住跟读",width-dp(93),candidateTop-dp(12),13,0xff146b54);hit(new RectF(width-dp(105),candidateTop-dp(38),width,candidateTop),"speech-follow");
        }
        JSONArray candidates = rows(); float x = dp(9); float y = candidateTop;
        visibleCandidates = 0;
        for (int index = candidateOffset; index < candidates.length(); index++) {
            JSONObject candidate = candidates.optJSONObject(index); if (candidate == null) continue;
            String value = candidate.optString("text");
            paint.setTextSize(dp(19)); float itemWidth = Math.min(width * .7f, paint.measureText(value) + dp(27));
            if (x + itemWidth > width && visibleCandidates > 0) break;
            paint.setColor(index == 0 ? 0xffdceadf : 0xffedf2ef);
            RectF bounds = new RectF(x, y, Math.min(width - dp(5), x + itemWidth), y + dp(39));
            canvas.drawRoundRect(bounds, dp(8), dp(8), paint);
            canvas.save(); canvas.clipRect(bounds);
            label(canvas, value, x + dp(10), y + dp(27), 19, 0xff143d30); canvas.restore();
            candidateBounds.add(bounds); candidateIndices.add(index); visibleCandidates++; x += itemWidth + dp(4);
        }
        if (candidates.length() == 0) label(canvas, "输入拼音，选择中文或整句译文", dp(12), y + dp(26), 13, 0xff647b70);
        String[] rows = symbols ? new String[]{"1 2 3 4 5 6 7 8 9 0", "@ # $ % & - + ( )", "ABC ! ? : ; / \" ⌫", "中英 🌐 , 空格 . ↵"} : t9 && !english ?
            new String[]{"符号 2·ABC 3·DEF ⌫", "4·GHI 5·JKL 6·MNO 中英", "7·PQRS 8·TUV 9·WXYZ 🌐", "123 26键 , 空格 . ↵"} :
            new String[]{"q w e r t y u i o p", "a s d f g h j k l", "⇧ z x c v b n m ⌫", "123 中英 🌐 , 空格 . ↵"};
        float top = candidateTop + dp(45);
        for (int row = 0; row < rows.length; row++) {
            String[] values = rows[row].split(" ");
            float units = 0; for (String value : values) units += value.equals("空格") ? 3.4f : 1;
            float inset = (!symbols && !t9 && row == 1) ? width * .045f : dp(3);
            float unit = (width - inset * 2) / units; x = inset;
            for (String value : values) {
                float size = unit * (value.equals("空格") ? 3.4f : 1);
                RectF bounds = new RectF(x + dp(2), top + dp(3), x + size - dp(2), top + dp(46));
                paint.setColor(value.equals("空格") ? 0xffd7e7dc : 0xfffbfdfb);
                canvas.drawRoundRect(bounds, dp(7), dp(7), paint);
                String shown = value.equals("空格") && !input.isEmpty() ? "选择中文" : value;
                if (shift && shown.length() == 1) shown = shown.toUpperCase(java.util.Locale.ROOT);
                paint.setColor(0xff143d30); paint.setTextSize(dp(shown.length() > 1 ? 13 : 21));
                paint.setTextAlign(Paint.Align.CENTER);
                canvas.drawText(shown, bounds.centerX(), bounds.centerY() - (paint.ascent() + paint.descent()) / 2, paint);
                hit(bounds, value); x += size;
            }
            top += dp(49);
        }
    }
    private void hit(RectF bounds, String value) { keyBounds.add(bounds); keyValues.add(value); }
    @Override public boolean onTouchEvent(MotionEvent event) {
        if (event.getAction() == MotionEvent.ACTION_DOWN) {
            touchX = event.getX(); touchY = event.getY();
            heldKey="";voiceHeld=false;
            for(int i=0;i<keyBounds.size();i++)if(keyBounds.get(i).contains(touchX,touchY))heldKey=keyValues.get(i);
            if(heldKey.equals("空格"))holdHandler.postDelayed(longSpace,350);
            if(heldKey.equals("speech-follow")){voiceHeld=true;listener.voice("follow");}
            performHapticFeedback(android.view.HapticFeedbackConstants.KEYBOARD_TAP); return true;
        }
        if (event.getAction() == MotionEvent.ACTION_UP) {
            holdHandler.removeCallbacks(longSpace);
            if(voiceHeld){voiceHeld=false;listener.voice("stop");return true;}
            if (touchY >= candidateTop && touchY < candidateTop + dp(40) && Math.abs(event.getX() - touchX) > dp(40)) {
                candidateOffset = event.getX() < touchX ? Math.min(Math.max(0, rows().length() - 1), candidateOffset + Math.max(1, visibleCandidates)) :
                    Math.max(0, candidateOffset - Math.max(1, visibleCandidates));
                invalidate(); return true;
            }
            for (int index = 0; index < candidateBounds.size(); index++) if (candidateBounds.get(index).contains(event.getX(), event.getY())) {
                listener.candidate(candidateIndices.get(index), false); performClick(); return true;
            }
            for (int index = 0; index < keyBounds.size(); index++) if (keyBounds.get(index).contains(event.getX(), event.getY())) {
                listener.key(keyValues.get(index)); performClick(); return true;
            }
            return true;
        }
        if(event.getAction()==MotionEvent.ACTION_CANCEL){holdHandler.removeCallbacks(longSpace);if(voiceHeld)listener.voice("cancel");voiceHeld=false;}
        if(event.getAction()==MotionEvent.ACTION_MOVE&&Math.abs(event.getY()-touchY)>dp(60)){holdHandler.removeCallbacks(longSpace);if(voiceHeld){listener.voice("cancel");voiceHeld=false;}}
        return true;
    }
    @Override public boolean performClick() { super.performClick(); return true; }
    @Override protected void onDetachedFromWindow(){holdHandler.removeCallbacks(longSpace);if(voiceHeld)listener.voice("cancel");super.onDetachedFromWindow();}
}
