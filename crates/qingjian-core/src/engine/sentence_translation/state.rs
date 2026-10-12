//! 整句翻译的请求代次与有界内存缓存，不写入个人词语释义表。

use std::collections::VecDeque;

use super::{SentenceTranslationRequest, SentenceTranslator};
use crate::{Language, Translation};

#[derive(Default)]
pub struct SentenceTranslationState {
    pub provider: Option<Box<dyn SentenceTranslator>>,

    pub sequence: u64,

    pub pending: Option<SentenceTranslationRequest>,

    /// 仅当前进程保存，重配 AI 或切换风格时清空。
    pub cache: VecDeque<(Language, String, Translation)>,
}

impl SentenceTranslationState {
    pub fn cancel(&mut self) {
        self.sequence += 1;
        self.pending = None;
        if let Some(provider) = self.provider.as_mut() {
            provider.cancel();
        }
    }

    pub fn translation(&self, language: Language, text: &str) -> Option<Translation> {
        self.cache
            .iter()
            .find(|(lang, source, _)| *lang == language && source == text)
            .map(|(_, _, translation)| translation.clone())
    }
}
