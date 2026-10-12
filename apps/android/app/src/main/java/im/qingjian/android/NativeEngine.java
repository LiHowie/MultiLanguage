// Android JNI 门面；词库、组句、学习和翻译均由 Rust Engine 处理。
package im.modolanguage.android;

final class NativeEngine {
    static native String membershipRequest(String config, String route, String body);
    static { System.loadLibrary("qingjian_android"); }
    static native long create(String data, String user, String config);
    static native String command(long handle, String action, String value);
    static native String configure(long handle, String config);
    static native void destroy(long handle);
    static native String testConnection(String config);
    private NativeEngine() { }
}
