//! 整句翻译的异步提供方；网络实现放在 predict crate。

use super::{SentenceTranslationRequest, SentenceTranslationResult};

pub trait SentenceTranslator: Send {
    /// 只入队，不等待网络。
    fn submit(&mut self, request: SentenceTranslationRequest);

    /// 作废尚未发送的请求。
    fn cancel(&mut self);

    /// 只取已经完成的结果。
    fn poll(&mut self) -> Option<SentenceTranslationResult>;
}
