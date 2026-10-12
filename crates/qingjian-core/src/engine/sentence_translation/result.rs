//! 整句翻译结果；空译文表示请求失败，可以继续正常输入。

#[derive(Debug, Clone)]
pub struct SentenceTranslationResult {
    pub sequence: u64,

    pub text: Option<String>,
}
