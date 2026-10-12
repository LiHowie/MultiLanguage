//! 九键词目，数字码由标准电话键盘映射生成。
use crate::{Candidate, CandidateKind};

pub(super) struct Entry {
    pub code: String,

    pub text: String,

    pub syllables: Vec<String>,

    pub frequency: u32,
}

impl Entry {
    pub fn candidate(&self) -> Candidate {
        Candidate {
            text: self.text.clone(),
            kind: CandidateKind::Chinese,
            syllables: self.syllables.clone(),
            reading: None,
            translation: None,
            aux_code: None,
        }
    }
}
