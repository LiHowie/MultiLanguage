//! 魔多学习服务控制器；录音属于平台层，翻译与纠错请求通过共用服务接口。
mod audio;
mod message;
mod state;
pub(crate) use message::LearningMessage;
pub(super) use state::LearningState;

use super::{Message, Settings};
use base64::Engine as Base64Engine;
use qingjian_predict::MembershipClient;
use serde_json::{Value, json};
use std::io::Write;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows_reactor::ComponentContext;

impl Settings {
    pub(super) fn learning_request(
        &mut self,
        action: &'static str,
        route: &'static str,
        body: Option<Value>,
        context: &ComponentContext<Self>,
    ) {
        if self.learning.busy {
            return;
        }
        let mut config = self.config.predict.clone();
        config.base_url = self.learning.url.clone();
        let client = match MembershipClient::new(&config) {
            Ok(c) => c,
            Err(e) => {
                self.learning.notice = e;
                return;
            }
        };
        self.learning.busy = true;
        self.learning.generation += 1;
        let generation = self.learning.generation;
        self.learning.notice = "正在处理…".to_owned();
        context.spawn_background(move |_cancel| {
            Message::Membership(LearningMessage::Done(
                generation,
                action,
                client.block(route, body),
            ))
        });
    }

    pub(super) fn cancel_learning(&mut self) {
        self.learning.generation += 1;
        if let Some(discard) = self.learning.discard.take() {
            discard.store(true, Ordering::Relaxed);
        }
        if let Some(stop) = self.learning.stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.learning.busy = false;
        self.learning.notice = "操作已取消".to_owned();
    }

    pub(super) fn update_learning(
        &mut self,
        message: LearningMessage,
        context: &ComponentContext<Self>,
    ) {
        match message {
            LearningMessage::Email(v) => self.learning.email = v,
            LearningMessage::Password(v) => self.learning.password = v,
            LearningMessage::Url(v) => self.learning.url = v,
            LearningMessage::Consent(v) => {
                if !v {
                    self.cancel_learning();
                }
                self.save("predict", "cloud_consent", v);
            }
            LearningMessage::Mode(Some(i)) => {
                self.cancel_learning();
                self.learning.learning_mode = i == 1;
            }
            LearningMessage::Cancel => self.cancel_learning(),
            LearningMessage::Logout => {
                self.cancel_learning();
                self.learning_request("logout", "/auth/logout", Some(json!({})), context);
                self.save("predict", "api_key", "");
                self.learning.speech = Value::Null;
                self.learning.password.clear();
                if let Some(root) = qingjian_platform::dirs::user_dir() {
                    let _ = std::fs::remove_file(root.join("last-speech.json"));
                }
            }
            LearningMessage::Login(register) => {
                let body = json!({"email":self.learning.email,"password":self.learning.password});
                self.learning.password.clear();
                self.learning_request(
                    "login",
                    if register {
                        "/auth/register"
                    } else {
                        "/auth/login"
                    },
                    Some(body),
                    context,
                );
            }
            LearningMessage::Refresh => self.learning_request("me", "/me", None, context),
            LearningMessage::Pull => self.learning_request("pull", "/sync", None, context),
            LearningMessage::Sync => {
                let body = json!({"baseRevision":self.learning.revision,"cards":[],"preferences":{
                    "targetLanguage":self.config.general.learning_language,"inputLanguage":"zh","layout":"qwerty",
                    "voiceMode":if self.learning.learning_mode {"learning"}else{"native"}}});
                self.learning_request("sync", "/sync", Some(body), context);
            }
            LearningMessage::Stop => {
                if let Some(stop) = &self.learning.stop {
                    stop.store(true, Ordering::Relaxed);
                    self.learning.notice = "录音结束，正在识别…".to_owned();
                }
            }
            LearningMessage::Record => {
                if self.learning.busy {
                    return;
                }
                if !self.config.predict.cloud_consent {
                    self.learning.notice = "请先同意云端学习".to_owned();
                    return;
                }
                if self.config.predict.resolve_api_key().is_none() {
                    self.learning.notice = "请先登录会员账号".to_owned();
                    return;
                }
                let mut config = self.config.predict.clone();
                config.base_url = self.learning.url.clone();
                let client = match MembershipClient::new(&config) {
                    Ok(c) => c,
                    Err(e) => {
                        self.learning.notice = e;
                        return;
                    }
                };
                let stop = Arc::new(AtomicBool::new(false));
                let discard = Arc::new(AtomicBool::new(false));
                self.learning.stop = Some(stop.clone());
                self.learning.discard = Some(discard.clone());
                self.learning.busy = true;
                self.learning.generation += 1;
                let generation = self.learning.generation;
                self.learning.notice =
                    "正在录音；点击停止后识别，最长 30 秒。跟读时请看下方正确表达。".to_owned();
                let target = self.config.general.learning_language.clone();
                let mode = if self.learning.learning_mode {
                    "learning"
                } else {
                    "native"
                };
                context.spawn_background(move |cancel| {
                    let result=audio::record(&stop,&discard,&cancel).and_then(|wav| {
                        if discard.load(Ordering::Relaxed) || cancel.is_cancelled() { return Err("录音已取消".to_owned()); }
                        client.block("/voice",Some(json!({"audioBase64":base64::engine::general_purpose::STANDARD.encode(wav),"mode":mode,"target":target,"cloudConsent":true})))
                    });
                    Message::Membership(LearningMessage::Done(generation,"voice",result))
                });
            }
            LearningMessage::Prepare => {
                let text = self.learning.speech["output"]
                    .as_str()
                    .or_else(|| self.learning.speech["correctedText"].as_str())
                    .unwrap_or("");
                if text.is_empty() {
                    self.learning.notice = "先完成一次语音识别".to_owned();
                    return;
                }
                let Some(root) = qingjian_platform::dirs::user_dir() else {
                    return;
                };
                let expires = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    + 180;
                let payload = json!({"text":text,"expires":expires}).to_string();
                let result = qingjian_core::storage::write_atomic_private(
                    &root.join("last-speech.json"),
                    |writer| writer.write_all(payload.as_bytes()),
                );
                self.learning.notice = if result.is_ok() {
                    "已准备输出。回到输入框按 F8 插入，3 分钟内有效；私密输入框不会插入。"
                        .to_owned()
                } else {
                    "语音结果保存失败".to_owned()
                };
            }
            LearningMessage::SaveCard => {
                let speech = &self.learning.speech;
                if speech["correctedText"].as_str().is_none() {
                    self.learning.notice = "先完成一次外语学习录音".to_owned();
                    return;
                }
                let id = format!(
                    "voice-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                );
                let body = json!({"baseRevision":self.learning.revision,"cards":[{"id":id,"input":speech["correctedText"],"translation":speech["motherMeaning"],"reading":speech["reading"],"target":speech["target"],"deleted":false}]});
                self.learning_request("sync", "/sync", Some(body), context);
            }
            LearningMessage::Done(generation, action, result) => {
                if generation != self.learning.generation {
                    return;
                }
                self.learning.busy = false;
                self.learning.stop = None;
                self.learning.discard = None;
                match result {
                    Err(error) => self.learning.notice = error,
                    Ok(value) => match action {
                        "login" => {
                            let Some(token) = value["token"].as_str() else {
                                self.learning.notice = "账号响应缺少令牌".to_owned();
                                return;
                            };
                            let url = self.learning.url.clone();
                            self.save("predict", "base_url", url);
                            self.save("predict", "membership_service", true);
                            self.save("predict", "api_key", token);
                            self.learning.notice = format!(
                                "已登录 {}。请刷新会员状态并选择是否启用整句翻译。",
                                value["user"]["email"].as_str().unwrap_or("")
                            );
                        }
                        "me" => {
                            self.learning.notice = format!(
                                "{} · {}\n本期翻译 {} / {} 次，语音 {} / {} 秒",
                                value["user"]["email"].as_str().unwrap_or(""),
                                if value["user"]["plan"] == "pro" {
                                    "会员"
                                } else {
                                    "基础版"
                                },
                                value["usage"]["translations"],
                                value["usage"]["limits"]["translations"],
                                value["usage"]["seconds"],
                                value["usage"]["limits"]["seconds"]
                            )
                        }
                        "logout" => self.learning.notice = "已退出账号，本机令牌已清除".to_owned(),
                        "voice" => {
                            self.learning.speech = value;
                            self.learning.notice =
                                "完成 · AI 辅助。可再次录音跟读，或准备插入语音结果。".to_owned();
                        }
                        "pull" | "sync" => {
                            self.learning.revision = value["revision"].as_u64().unwrap_or(0);
                            if action == "pull" {
                                if let Some(language) =
                                    value["preferences"]["targetLanguage"].as_str()
                                {
                                    self.save("general", "learning_language", language);
                                }
                                self.learning.learning_mode =
                                    value["preferences"]["voiceMode"] == "learning";
                            }
                            let cards = value["cards"]
                                .as_array()
                                .map(|cards| {
                                    cards
                                        .iter()
                                        .filter(|c| c["deleted"] != true)
                                        .map(|c| {
                                            format!(
                                                "{} → {}",
                                                c["input"].as_str().unwrap_or(""),
                                                c["translation"].as_str().unwrap_or("")
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                                .unwrap_or_default();
                            self.learning.notice = format!(
                                "学习记录已同步（版本 {}）\n{}",
                                self.learning.revision, cards
                            );
                        }
                        _ => {}
                    },
                }
            }
            _ => {}
        }
    }
}
