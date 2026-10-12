//! JNI 调用边界；句柄登记避免 Java 侧生命周期变动造成悬空指针。

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use qingjian_predict::ConnectionTest;

use crate::session::Session;
use crate::settings::Settings;

static SESSIONS: OnceLock<Mutex<HashMap<i64, Session>>> = OnceLock::new();
static NEXT: AtomicI64 = AtomicI64::new(1);

fn sessions() -> &'static Mutex<HashMap<i64, Session>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn input(env: &mut JNIEnv, text: &JString) -> Result<String, String> {
    env.get_string(text)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

fn guarded<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(operation))
        .map_err(|_| "引擎遇到内部错误，请关闭后重新打开键盘".to_owned())?
}

fn result(env: &mut JNIEnv, outcome: Result<String, String>) -> jstring {
    match outcome.and_then(|text| env.new_string(text).map_err(|error| error.to_string())) {
        Ok(text) => text.into_raw(),
        Err(error) => {
            let _ = env.throw_new("java/lang/IllegalStateException", error);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_create(
    mut env: JNIEnv,
    _class: JClass,
    data: JString,
    user: JString,
    config: JString,
) -> jlong {
    let outcome = guarded(|| {
        let data = input(&mut env, &data)?;
        let user = input(&mut env, &user)?;
        let config = input(&mut env, &config)?;
        let settings = serde_json::from_str(&config).map_err(|error| error.to_string())?;
        let session = Session::new(Path::new(&data), Path::new(&user), settings)?;
        let handle = NEXT.fetch_add(1, Ordering::Relaxed);
        sessions()
            .lock()
            .map_err(|error| error.to_string())?
            .insert(handle, session);
        Ok::<_, String>(handle)
    });
    match outcome {
        Ok(handle) => handle,
        Err(error) => {
            let _ = env.throw_new("java/lang/IllegalStateException", error);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_command(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    action: JString,
    value: JString,
) -> jstring {
    let outcome = guarded(|| {
        let action = input(&mut env, &action)?;
        let value = input(&mut env, &value)?;
        let mut sessions = sessions().lock().map_err(|error| error.to_string())?;
        let session = sessions.get_mut(&handle).ok_or("输入会话已结束")?;
        serde_json::to_string(&session.command(&action, &value)).map_err(|error| error.to_string())
    });
    result(&mut env, outcome)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_configure(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    config: JString,
) -> jstring {
    let outcome = guarded(|| {
        let config = input(&mut env, &config)?;
        let settings = serde_json::from_str(&config).map_err(|error| error.to_string())?;
        let mut sessions = sessions().lock().map_err(|error| error.to_string())?;
        let session = sessions.get_mut(&handle).ok_or("输入会话已结束")?;
        session.configure(settings)?;
        serde_json::to_string(&session.command("poll", "")).map_err(|error| error.to_string())
    });
    result(&mut env, outcome)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_destroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if let Ok(mut sessions) = sessions().lock() {
        sessions.remove(&handle);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_testConnection(
    mut env: JNIEnv,
    _class: JClass,
    config: JString,
) -> jstring {
    let outcome = guarded(|| {
        let config = input(&mut env, &config)?;
        let settings: Settings =
            serde_json::from_str(&config).map_err(|error| error.to_string())?;
        let test = ConnectionTest::start(&settings.predict).map_err(|error| error.to_string())?;
        let started = Instant::now();
        loop {
            if let Some(report) = test.poll() {
                return report
                    .map(|report| {
                        format!(
                            "连接成功：{}，{} ms",
                            report.model,
                            report.elapsed.as_millis()
                        )
                    })
                    .map_err(|error| error.to_string());
            }
            if started.elapsed()
                > Duration::from_millis(settings.predict.timeout_ms.saturating_add(1000))
            {
                return Err("连接超时，请检查接口地址和网络".to_owned());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    });
    result(&mut env, outcome)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_im_modolanguage_android_NativeEngine_membershipRequest(
    mut env: JNIEnv,
    _class: JClass,
    config: JString,
    route: JString,
    body: JString,
) -> jstring {
    let outcome = guarded(|| {
        let settings: Settings =
            serde_json::from_str(&input(&mut env, &config)?).map_err(|error| error.to_string())?;
        let route = input(&mut env, &route)?;
        if !matches!(
            route.as_str(),
            "/auth/register" | "/auth/login" | "/auth/logout" | "/me" | "/sync" | "/voice"
        ) {
            return Err("不支持的学习操作".to_owned());
        }
        let body = input(&mut env, &body)?;
        let body = if body.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&body).map_err(|error| error.to_string())?)
        };
        let client = qingjian_predict::MembershipClient::new(&settings.predict)?;
        client.block(&route, body).map(|value| value.to_string())
    });
    result(&mut env, outcome)
}
