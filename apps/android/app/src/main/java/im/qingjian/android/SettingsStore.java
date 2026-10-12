// 普通配置存于应用私有偏好；密钥使用 Android Keystore 的 AES-GCM 加密。
package im.modolanguage.android;

import android.content.Context;
import android.content.SharedPreferences;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;
import org.json.JSONObject;
import java.nio.charset.StandardCharsets;
import java.security.KeyStore;
import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

final class SettingsStore {
    private static final String ALIAS = "modo-session-token";
    private final SharedPreferences preferences;

    SettingsStore(Context context) {
        preferences = context.getSharedPreferences("modo", Context.MODE_PRIVATE);
    }
    String language() { return preferences.getString("language", "en"); }
    String style() { return preferences.getString("style", "conversational"); }
    String url() { return preferences.getString("url", "http://10.0.2.2:8797/v1"); }
    String model() { return "modo-membership"; }
    String layout() { return preferences.getString("layout", "qwerty"); }
    String voiceMode() { return preferences.getString("voice_mode", "native"); }
    boolean consent() { return preferences.getBoolean("consent", false); }
    void local(String language, String style, String layout, String mode, boolean consent, boolean enabled) {
        preferences.edit().putString("language", language).putString("style", style).putString("layout", layout)
            .putString("voice_mode", mode).putBoolean("consent", consent).putBoolean("enabled", enabled).apply();
    }
    void account(String url, String token) throws Exception { save(language(), style(), url, model(), token, enabled()); }
    boolean enabled() { return preferences.getBoolean("enabled", false); }

    private SecretKey key() throws Exception {
        KeyStore store = KeyStore.getInstance("AndroidKeyStore");
        store.load(null);
        if (!store.containsAlias(ALIAS)) {
            KeyGenerator generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore");
            generator.init(new KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE).build());
            generator.generateKey();
        }
        return (SecretKey) store.getKey(ALIAS, null);
    }
    String apiKey() throws Exception {
        String encrypted = preferences.getString("key_cipher", "");
        if (encrypted.isEmpty()) return "";
        byte[] iv = Base64.decode(preferences.getString("key_iv", ""), Base64.NO_WRAP);
        Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
        cipher.init(Cipher.DECRYPT_MODE, key(), new GCMParameterSpec(128, iv));
        return new String(cipher.doFinal(Base64.decode(encrypted, Base64.NO_WRAP)), StandardCharsets.UTF_8);
    }
    void save(String language, String style, String url, String model, String apiKey, boolean enabled) throws Exception {
        SharedPreferences.Editor editor = preferences.edit().putString("language", language).putString("style", style)
            .putString("url", url.trim()).putString("model", model.trim()).putBoolean("enabled", enabled);
        if (apiKey.trim().isEmpty()) {
            editor.remove("key_cipher").remove("key_iv");
        } else {
            Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
            cipher.init(Cipher.ENCRYPT_MODE, key());
            editor.putString("key_cipher", Base64.encodeToString(cipher.doFinal(apiKey.trim().getBytes(StandardCharsets.UTF_8)), Base64.NO_WRAP));
            editor.putString("key_iv", Base64.encodeToString(cipher.getIV(), Base64.NO_WRAP));
        }
        if (!editor.commit()) throw new IllegalStateException("设置保存失败");
    }
    String json() throws Exception {
        JSONObject predict = new JSONObject().put("membership_service", true).put("cloud_consent", consent())
            .put("sentence_translation", enabled() && consent()).put("enabled", false)
            .put("base_url", url()).put("model", model()).put("api_key", apiKey())
            .put("reasoning_effort", "none").put("timeout_ms", 15000)
            .put("translation_debounce_ms", 800).put("translation_style", style());
        return new JSONObject().put("learning_language", language()).put("predict", predict).toString();
    }
}
