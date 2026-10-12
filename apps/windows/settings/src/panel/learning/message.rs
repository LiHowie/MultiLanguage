//! 魔多账号与语音学习页面的消息。
#[derive(Clone)]
pub(crate) enum LearningMessage {
    Email(String),
    Password(String),
    Url(String),
    Consent(bool),
    Login(bool),
    Logout,
    Refresh,
    Pull,
    Sync,
    Record,
    Stop,
    Cancel,
    Mode(Option<usize>),
    SaveCard,
    Prepare,
    Done(u64, &'static str, Result<serde_json::Value, String>),
}
