// 开发包端到端检查：JNI、真实产品词库、26/9键、会员权限与语音结果，不调用外部模型。
package im.modolanguage.android;
import android.app.Activity;
import android.app.Instrumentation;
import android.os.Bundle;
import android.content.Context;
import android.text.InputType;
import org.json.JSONObject;
import org.json.JSONArray;
import java.io.File;
import java.util.UUID;

public final class ModoSmokeInstrumentation extends Instrumentation {
    private Bundle arguments;
    @Override public void onCreate(Bundle value){arguments=value;super.onCreate(value);start();}
    @Override public void onStart(){Bundle result=new Bundle();long handle=0;try{
        Context context=getTargetContext();SettingsStore store=new SettingsStore(context);
        File data=ProductData.prepare(context);String config=store.json();
        handle=NativeEngine.create(data.getAbsolutePath(),new File(context.getFilesDir(),"smoke-user").getAbsolutePath(),config);
        JSONObject frame=new JSONObject(NativeEngine.command(handle,"push","nihao"));
        check(has(frame,"你好"),"26键未命中你好");int position=find(frame,"你好");
        check(new JSONObject(NativeEngine.command(handle,"select",Integer.toString(position))).getString("commit").equals("你好"),"26键提交失败");
        frame=new JSONObject(NativeEngine.command(handle,"push9","64426"));check(has(frame,"你好"),"9键未命中你好："+frame);
        check(new JSONObject(NativeEngine.command(handle,"select",Integer.toString(find(frame,"你好")))).getString("commit").equals("你好"),"9键提交失败");
        NativeEngine.command(handle,"push9","64426");frame=new JSONObject(NativeEngine.command(handle,"backspace",""));check(frame.getString("input").equals("6442"),"9键退格错误");
        frame=new JSONObject(NativeEngine.command(handle,"reset","private"));check(frame.getBoolean("private")&&!frame.getBoolean("translation_enabled"),"隐私边界失效");
        check(ModoImeService.isPassword(InputType.TYPE_CLASS_TEXT|InputType.TYPE_TEXT_VARIATION_PASSWORD),"密码框判断错误");
        String endpoint=arguments.getString("endpoint","");
        if(!endpoint.isEmpty()){
            JSONObject cfg=new JSONObject(config);JSONObject predict=cfg.getJSONObject("predict");predict.put("base_url",endpoint).put("cloud_consent",true).put("api_key","");
            JSONObject free=new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/auth/register",new JSONObject().put("email",UUID.randomUUID()+"@smoke.example").put("password","modo-smoke-password").toString()));
            check(free.getJSONObject("user").getString("plan").equals("free"),"注册可绕过会员");
            predict.put("api_key",free.getString("token"));
            JSONObject me=new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/me",""));check(me.getJSONObject("user").getString("plan").equals("free"),"免费账号状态错误");
            JSONObject account=new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/auth/login","{\"email\":\"member@modo.example\",\"password\":\"modo-smoke-password\"}"));predict.put("api_key",account.getString("token")).put("sentence_translation",true).put("translation_debounce_ms",100);
            check(new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/me","")).getJSONObject("user").getString("plan").equals("pro"),"会员登录失败");
            NativeEngine.configure(handle,cfg.toString());NativeEngine.command(handle,"reset","");
            frame=new JSONObject(NativeEngine.command(handle,"push","woxiangxuexiwaiguoyu"));
            check(frame.getJSONArray("candidates").getJSONObject(0).getBoolean("sentence"),"产品词库未生成整句");
            long until=System.currentTimeMillis()+12000;boolean translated=false;
            do{Thread.sleep(100);frame=new JSONObject(NativeEngine.command(handle,"poll",""));if(frame.getJSONArray("candidates").getJSONObject(0).optString("translation").equals("I want to learn a foreign language.")){translated=true;break;}}while(System.currentTimeMillis()<until);
            check(translated,"会员整句译文未出现："+frame);
            JSONArray cards=new JSONArray().put(new JSONObject().put("id",UUID.randomUUID().toString()).put("input","hello").put("translation","你好").put("target","en").put("deleted",false));
            JSONObject sync=new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/sync",""));JSONObject saved=new JSONObject(NativeEngine.membershipRequest(cfg.toString(),"/sync",new JSONObject().put("baseRevision",sync.getLong("revision")).put("cards",cards).toString()));check(saved.getJSONArray("cards").length()>0,"学习卡片同步失败");
        }
        result.putString("stream","PASS: 产品词库、26键、9键、提交、退格、密码隐私、会员登录、整句翻译、卡片同步\n");finish(Activity.RESULT_OK,result);
    }catch(Throwable error){result.putString("stream","FAIL: "+error+"\n");finish(Activity.RESULT_CANCELED,result);}finally{if(handle!=0)NativeEngine.destroy(handle);}}
    private boolean has(JSONObject frame,String text)throws Exception{return find(frame,text)>=0;}
    private int find(JSONObject frame,String text)throws Exception{JSONArray rows=frame.getJSONArray("candidates");for(int i=0;i<rows.length();i++)if(rows.getJSONObject(i).getString("text").equals(text))return i;return -1;}
    private void check(boolean value,String error){if(!value)throw new AssertionError(error);}
}
