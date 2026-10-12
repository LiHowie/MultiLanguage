// 录音生命周期与 Core 服务桥；取消、切换输入框和隐私状态会作废旧结果。
package im.modolanguage.android;
import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import android.os.Handler;
import android.os.Looper;
import android.util.Base64;
import org.json.JSONObject;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

final class VoiceTeacher {
    interface Listener { void result(JSONObject value); void message(String value); }
    private final Context context;
    private final Listener listener;
    private final ExecutorService worker=Executors.newSingleThreadExecutor();
    private final Handler main=new Handler(Looper.getMainLooper());
    private VoiceRecorder recorder;
    private volatile int generation;
    VoiceTeacher(Context context,Listener listener) {this.context=context;this.listener=listener;}
    void start() {
        if(recorder!=null)return;
        if(context.checkSelfPermission(Manifest.permission.RECORD_AUDIO)!=PackageManager.PERMISSION_GRANTED){listener.message("请先在魔多设置中允许麦克风");return;}
        try {
            SettingsStore settings=new SettingsStore(context);
            if(!settings.consent())throw new IllegalStateException("请先在设置中同意云端学习");
            if(settings.apiKey().isEmpty())throw new IllegalStateException("请先登录魔多会员账号");
            String config=settings.json(),mode=settings.voiceMode(),target=settings.language();
            int id=++generation;VoiceRecorder current=new VoiceRecorder();recorder=current;listener.message("正在录音，松开后提交…");
            worker.execute(()->{
                try {
                    byte[] audio=current.record();if(id!=generation)return;
                    main.post(()->{if(id==generation)listener.message("正在识别…");});
                    String body=new JSONObject().put("audioBase64",Base64.encodeToString(audio,Base64.NO_WRAP))
                        .put("mode",mode).put("target",target).put("cloudConsent",true).toString();
                    JSONObject value=new JSONObject(NativeEngine.membershipRequest(config,"/voice",body));
                    main.post(()->{if(id==generation){recorder=null;listener.result(value);}});
                } catch(Exception|LinkageError error){main.post(()->{if(id==generation){recorder=null;listener.message(error.getMessage());}});}
            });
        } catch(Exception error){listener.message(error.getMessage());}
    }
    void stop(boolean cancel) {if(cancel)++generation;if(recorder!=null)recorder.stop();if(cancel)recorder=null;}
    void close(){stop(true);worker.shutdown();}
}
