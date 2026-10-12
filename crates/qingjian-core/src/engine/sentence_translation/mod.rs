//! 整句候选的云端译文：异步请求、过期结果过滤、仅内存缓存。

mod provider;
mod request;
mod result;
mod state;

pub use provider::SentenceTranslator;
pub use request::SentenceTranslationRequest;
pub use result::SentenceTranslationResult;
pub(super) use state::SentenceTranslationState;

use super::Engine;
use crate::{Candidate, CandidateKind, Language, Sense, Translation};

/// 避免逐字短词与超长缓冲区触发整句请求。
const MIN_CHARS: usize = 2;
const MAX_CHARS: usize = 500;
const CACHE_CAPACITY: usize = 64;

impl Engine {
    /// 启用或重配整句翻译；配置、模型、风格变化时旧请求与缓存全部作废。
    pub fn set_sentence_translator(&mut self, provider: Option<Box<dyn SentenceTranslator>>) {
        self.sentence_translation.cancel();
        self.sentence_translation.cache.clear();
        self.sentence_translation.provider = provider;
    }

    /// 给当前整句候选请求译文。输入法壳在缓冲变化、候选重排或高亮变化后调，不阻塞。
    /// 只发候选原文；私密输入与关闭学习语言时不发。
    pub fn request_sentence_translation(&mut self, candidates: &[Candidate]) {
        let language = self.translator.language();
        let candidate = candidates.iter().find(|candidate| {
            candidate.kind == CandidateKind::Sentence
                && (MIN_CHARS..=MAX_CHARS).contains(&candidate.text.chars().count())
        });
        if self.private || language == Language::Chinese {
            self.sentence_translation.cancel();
            return;
        }
        let Some(candidate) = candidate else {
            self.sentence_translation.cancel();
            return;
        };
        let state = &mut self.sentence_translation;
        let Some(provider) = state.provider.as_mut() else {
            return;
        };
        let text = &candidate.text;
        if state
            .pending
            .as_ref()
            .is_some_and(|pending| pending.language == language && pending.text == *text)
        {
            return;
        }
        if state
            .cache
            .iter()
            .any(|(lang, source, _)| *lang == language && source == text)
        {
            state.cancel();
            return;
        }
        state.sequence += 1;
        let request = SentenceTranslationRequest {
            sequence: state.sequence,
            text: text.clone(),
            language,
        };
        provider.submit(request.clone());
        state.pending = Some(request);
    }

    /// 接收译文，返回是否需要重绘。代次、语言与当前请求不一致的结果丢弃。
    pub fn poll_sentence_translation(&mut self) -> bool {
        let language = self.translator.language();
        let state = &mut self.sentence_translation;
        let Some(provider) = state.provider.as_mut() else {
            return false;
        };
        let mut changed = false;
        while let Some(result) = provider.poll() {
            let Some(pending) = state.pending.as_ref() else {
                continue;
            };
            if self.private || pending.language != language || pending.sequence != result.sequence {
                continue;
            }
            let Some(text) = result
                .text
                .filter(|text| !text.trim().is_empty() && text.chars().count() <= 4000)
            else {
                // 不永久缓存失败；重新输入这句话可重试。
                state.pending = None;
                continue;
            };
            let translation = Translation::new(
                language,
                vec![Sense {
                    part_of_speech: None,
                    text: text.trim().to_owned(),
                    reading: None,
                    fresh: false,
                }],
            );
            state
                .cache
                .push_back((language, pending.text.clone(), translation));
            if state.cache.len() > CACHE_CAPACITY {
                state.cache.pop_front();
            }
            state.pending = None;
            changed = true;
        }
        changed
    }
}
