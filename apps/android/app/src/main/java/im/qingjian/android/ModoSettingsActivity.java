// 魔多账号、键盘布局与语音老师；网络操作在 Core 服务桥的后台线程运行。
package im.modolanguage.android;
import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.provider.Settings;
import android.text.InputType;
import android.view.MotionEvent;
import android.widget.*;
import org.json.JSONObject;
import org.json.JSONArray;
import java.util.UUID;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class ModoSettingsActivity extends Activity {
    private final ExecutorService worker=Executors.newSingleThreadExecutor();
    private SettingsStore store;
    private LinearLayout content;
    private TextView status,speechText,accountText,cards;
    private EditText endpoint,email,password;
    private Spinner language,style,layout,voiceMode;
    private CheckBox consent,translation;
    private VoiceTeacher teacher;
    private JSONObject speech;
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);store=new SettingsStore(this);
        ScrollView scroll=new ScrollView(this);content=new LinearLayout(this);content.setOrientation(LinearLayout.VERTICAL);content.setPadding(28,24,28,32);scroll.addView(content);setContentView(scroll);
        scroll.setOnApplyWindowInsetsListener((v,insets)->{int top=android.os.Build.VERSION.SDK_INT>=30?insets.getInsets(android.view.WindowInsets.Type.systemBars()).top:insets.getSystemWindowInsetTop();int bottom=android.os.Build.VERSION.SDK_INT>=30?insets.getInsets(android.view.WindowInsets.Type.systemBars()).bottom:insets.getSystemWindowInsetBottom();v.setPadding(0,top,0,bottom);return insets;});
        label("魔多外语学习输入法",24);label("基础输入与本地单词翻译免费 · 学习会员 20 元/月",14);
        button("启用魔多键盘",()->startActivity(new Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)));
        button("选择输入法",()->((android.view.inputmethod.InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showInputMethodPicker());
        label("账号与会员",20);endpoint=input("魔多服务地址（含 /v1）",store.url(),false);email=input("邮箱","",false);password=input("密码，至少 10 位","",true);
        button("登录",()->login(false));button("注册",()->login(true));button("刷新会员",this::refresh);button("退出账号",()->call("/auth/logout",new JSONObject(),v->{store.account(store.url(),"");accountText.setText("已退出");cards.setText("");}));
        accountText=label("登录后显示会员状态。开发版不开放付款，测试会员由运营人员开通。",14);
        label("学习和键盘设置",20);
        language=spinner("学习语言",new String[]{"英语","日语"},store.language().equals("ja")?1:0);
        style=spinner("翻译风格",new String[]{"口语化","书面化"},store.style().equals("formal")?1:0);
        layout=spinner("中文键盘",new String[]{"26 键","9 键"},store.layout().equals("t9")?1:0);
        voiceMode=spinner("语音模式",new String[]{"母语输入：输出识别原文","外语学习：释义、纠错、跟读"},store.voiceMode().equals("learning")?1:0);
        consent=new CheckBox(this);consent.setText("同意将当前待翻译文字或主动录音发到魔多服务和 AI 服务商处理");consent.setChecked(store.consent());content.addView(consent);
        translation=new CheckBox(this);translation.setText("启用会员整句翻译");translation.setChecked(store.enabled());content.addView(translation);
        button("保存学习设置",()->{saveLocal();message("设置已保存，重新打开键盘生效");});
        button("同步学习设置",()->{saveLocal();call("/sync",null,v->{JSONObject prefs=new JSONObject().put("targetLanguage",store.language()).put("inputLanguage","zh").put("layout",store.layout()).put("voiceMode",store.voiceMode());call("/sync",new JSONObject().put("baseRevision",v.getLong("revision")).put("preferences",prefs).put("cards",new JSONArray()),r->message("已同步，其他设备可拉取"));});});
        button("从账号拉取学习设置",()->call("/sync",null,v->{JSONObject prefs=v.optJSONObject("preferences");if(prefs!=null){language.setSelection(prefs.optString("targetLanguage",store.language()).equals("ja")?1:0);layout.setSelection(prefs.optString("layout",store.layout()).equals("t9")?1:0);voiceMode.setSelection(prefs.optString("voiceMode",store.voiceMode()).equals("learning")?1:0);saveLocal();}message("已拉取，云端同意仍由本机单独选择");}));
        label("语音老师",20);button("允许麦克风",()->requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO},7));
        teacher=new VoiceTeacher(this,new VoiceTeacher.Listener(){public void result(JSONObject value){speech=value;showSpeech();}public void message(String value){ModoSettingsActivity.this.message(value);}});
        hold(button("按住说话（最长 30 秒）",()->{}),false);hold(button("显示正确句子并跟读",()->{}),true);
        speechText=label("外语学习模式会显示母语含义和正确表达；AI 不进行发音评分。",16);
        button("复制输出",()->{if(speech!=null){String value=speech.optString("output",speech.optString("correctedText"));((android.content.ClipboardManager)getSystemService(CLIPBOARD_SERVICE)).setPrimaryClip(android.content.ClipData.newPlainText("魔多语音",value));message("已复制，可返回输入框粘贴");}});
        button("收藏纠正句子",this::saveCard);label("学习卡片",20);button("拉取学习卡片",this::refresh);cards=label("只同步主动收藏的内容，不上传完整输入历史。",14);
        label("魔多客户端采用 GPL-3.0-or-later，基于青简；词库保留各自来源和许可。",13);
        button("源码与许可",()->startActivity(new Intent(Intent.ACTION_VIEW,android.net.Uri.parse("https://github.com/LiHowie/MultiLanguage"))));status=label("",14);
    }
    private TextView label(String value,int size){TextView v=new TextView(this);v.setText(value);v.setTextSize(size);v.setPadding(0,14,0,10);content.addView(v);return v;}
    private EditText input(String hint,String text,boolean secret){EditText v=new EditText(this);v.setHint(hint);v.setSingleLine(true);v.setText(text);if(secret)v.setInputType(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_VARIATION_PASSWORD);content.addView(v);return v;}
    private Button button(String text,Runnable action){Button v=new Button(this);v.setText(text);v.setOnClickListener(x->action.run());content.addView(v);return v;}
    private Spinner spinner(String title,String[] values,int selected){label(title,14);Spinner v=new Spinner(this);v.setAdapter(new ArrayAdapter<>(this,android.R.layout.simple_spinner_dropdown_item,values));v.setSelection(selected);content.addView(v);return v;}
    private void message(String value){if(status!=null)status.setText(value==null?"操作未完成":value);}
    private void saveLocal(){store.local(language.getSelectedItemPosition()==1?"ja":"en",style.getSelectedItemPosition()==1?"formal":"conversational",layout.getSelectedItemPosition()==1?"t9":"qwerty",voiceMode.getSelectedItemPosition()==1?"learning":"native",consent.isChecked(),translation.isChecked());}
    interface Reply {void result(JSONObject value)throws Exception;}
    private void call(String route,JSONObject body,Reply reply){worker.execute(()->{try{JSONObject v=new JSONObject(NativeEngine.membershipRequest(store.json(),route,body==null?"":body.toString()));runOnUiThread(()->{if(isFinishing()||isDestroyed())return;try{reply.result(v);}catch(Exception e){message(e.getMessage());}});}catch(Exception|LinkageError e){runOnUiThread(()->message(e.getMessage()));}});}
    private void login(boolean register){String service=endpoint.getText().toString().trim(),mail=email.getText().toString(),pass=password.getText().toString();password.setText("");worker.execute(()->{try{store.account(service,"");JSONObject b=new JSONObject().put("email",mail).put("password",pass);JSONObject v=new JSONObject(NativeEngine.membershipRequest(store.json(),register?"/auth/register":"/auth/login",b.toString()));store.account(service,v.getString("token"));runOnUiThread(this::refresh);}catch(Exception|LinkageError e){runOnUiThread(()->message(e.getMessage()));}});}
    private void refresh(){call("/me",null,v->{JSONObject u=v.getJSONObject("user"),usage=v.getJSONObject("usage");accountText.setText(u.getString("email")+" · "+(u.getString("plan").equals("pro")?"会员有效至 "+new java.util.Date(u.getLong("planUntil")):"基础版")+"\n本期翻译 "+usage.optInt("translations")+" 次，语音 "+usage.optInt("seconds")+" 秒");});call("/sync",null,v->{StringBuilder text=new StringBuilder();JSONArray list=v.getJSONArray("cards");for(int i=0;i<list.length();i++){JSONObject c=list.getJSONObject(i);if(!c.optBoolean("deleted"))text.append(c.optString("input")).append("\n").append(c.optString("translation")).append("\n\n");}cards.setText(text.length()==0?"暂无学习卡片":text.toString());});}
    private void hold(Button button,boolean follow){button.setOnTouchListener((v,e)->{if(e.getAction()==MotionEvent.ACTION_DOWN){saveLocal();if(follow&&speech!=null)speechText.setText("跟读："+speech.optString("correctedText")+"\n"+speech.optString("reading"));teacher.start();return true;}if(e.getAction()==MotionEvent.ACTION_UP){teacher.stop(false);return true;}if(e.getAction()==MotionEvent.ACTION_CANCEL){teacher.stop(true);return true;}return true;});}
    private void showSpeech(){speechText.setText(speech.optString("mode").equals("native")?speech.optString("output"):"原句："+speech.optString("transcript")+"\n含义："+speech.optString("motherMeaning")+"\n正确表达："+speech.optString("correctedText")+"\n"+speech.optString("reading")+"\n"+speech.optString("explanation"));message("完成 · AI 辅助");}
    private void saveCard(){if(speech==null||!speech.has("correctedText")){message("先完成一次外语学习录音");return;}call("/sync",null,v->{JSONObject c=new JSONObject().put("id",UUID.randomUUID().toString()).put("input",speech.getString("correctedText")).put("translation",speech.getString("motherMeaning")).put("reading",speech.optString("reading")).put("target",speech.optString("target",store.language())).put("deleted",false);call("/sync",new JSONObject().put("baseRevision",v.getLong("revision")).put("cards",new JSONArray().put(c)),r->message("已收藏并同步"));});}
    @Override protected void onPause(){if(teacher!=null)teacher.stop(true);super.onPause();}
    @Override protected void onDestroy(){if(teacher!=null)teacher.close();worker.shutdown();super.onDestroy();}
}
