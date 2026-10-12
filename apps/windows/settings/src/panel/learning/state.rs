//! 账号表单、会员信息与本次语音结果，密码仅驻留表单内存。
use std::sync::{Arc, atomic::AtomicBool};

pub(crate) struct LearningState {
    pub email: String,

    pub password: String,

    pub url: String,

    pub notice: String,

    pub busy: bool,

    pub learning_mode: bool,

    pub stop: Option<Arc<AtomicBool>>,

    pub discard: Option<Arc<AtomicBool>>,

    pub generation: u64,

    pub speech: serde_json::Value,

    pub revision: u64,
}

impl Default for LearningState {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            url: "http://127.0.0.1:8797/v1".to_owned(),
            notice: "基础功能免费；整句翻译和语音老师会员 20 元/月。开发版测试会员由运营人员开通。"
                .to_owned(),
            busy: false,
            learning_mode: false,
            stop: None,
            discard: None,
            generation: 0,
            speech: serde_json::Value::Null,
            revision: 0,
        }
    }
}
