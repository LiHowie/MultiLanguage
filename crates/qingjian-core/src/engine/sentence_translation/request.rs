//! 一次整句候选翻译请求，不带应用上下文。

use crate::Language;

#[derive(Debug, Clone)]
pub struct SentenceTranslationRequest {
    pub sequence: u64,

    /// 当前整句候选的原文。
    pub text: String,

    pub language: Language,
}
