//! 单条 Android 候选帧，仅传当前学习语言的一条译文。

use qingjian_core::{Candidate, CandidateKind};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Row {
    pub text: String,

    pub translation: Option<String>,

    pub sentence: bool,
}

impl From<&Candidate> for Row {
    fn from(candidate: &Candidate) -> Self {
        Self {
            text: candidate.text.clone(),
            translation: candidate
                .translation
                .as_ref()
                .and_then(|translation| translation.senses().first())
                .map(|sense| sense.text.clone()),
            sentence: candidate.kind == CandidateKind::Sentence,
        }
    }
}
