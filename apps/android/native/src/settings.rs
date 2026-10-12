//! Android 配置桥；AI 请求配置直接复用 PredictConfig。

use qingjian_predict::PredictConfig;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub learning_language: String,

    pub predict: PredictConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            learning_language: "en".to_owned(),
            predict: PredictConfig::default(),
        }
    }
}
