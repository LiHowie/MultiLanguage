//! Engine 返回给 Android 键盘的一帧，commit 只在实际提交时出现。

use serde::Serialize;

use crate::row::Row;

#[derive(Default, Serialize)]
pub(crate) struct Frame {
    pub input: String,

    pub candidates: Vec<Row>,

    pub commit: Option<String>,

    pub delete: bool,

    pub private: bool,

    pub translation_enabled: bool,

    pub error: Option<String>,
}
