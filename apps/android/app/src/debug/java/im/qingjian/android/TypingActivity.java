// 仅开发包提供真实编辑框，供系统键盘触摸与上屏验收。
package im.modolanguage.android;
import android.app.Activity;
import android.os.Bundle;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.EditText;
import android.view.inputmethod.InputMethodManager;

public final class TypingActivity extends Activity {
    @Override public void onCreate(Bundle state){super.onCreate(state);
        String layout=getIntent().getStringExtra("layout");if(layout!=null){SettingsStore s=new SettingsStore(this);s.local(s.language(),s.style(),layout,s.voiceMode(),s.consent(),s.enabled());}
        LinearLayout root=new LinearLayout(this);root.setOrientation(LinearLayout.VERTICAL);root.setPadding(24,120,24,24);TextView label=new TextView(this);label.setText("魔多输入验收 · 此页面只在开发包中提供");root.addView(label);EditText editor=new EditText(this);editor.setHint("输入你好，检查26键/9键选词和删除");editor.setMinLines(3);root.addView(editor);setContentView(root);editor.requestFocus();editor.postDelayed(()->((InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showSoftInput(editor,InputMethodManager.SHOW_IMPLICIT),500);}
}
