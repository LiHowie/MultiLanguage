//! Android 九键门面；解码、排序、组句与按候选消耗数字都留在 Core。
mod entry;
mod index;
mod path;
#[cfg(test)]
mod tests;

use self::index::{Index, encode};
use self::path::Path;
use super::{Engine, Learner};
use crate::{Candidate, CandidateKind, CandidateList};
use std::collections::HashSet;

pub struct Keypad {
    index: Index,

    digits: String,
}

impl Keypad {
    pub fn new(engine: &Engine) -> Self {
        Self {
            index: Index::new(engine),
            digits: String::new(),
        }
    }

    pub fn input(&self) -> &str {
        &self.digits
    }

    pub fn push(&mut self, digit: char) {
        if ('2'..='9').contains(&digit) && self.digits.len() < 96 {
            self.digits.push(digit);
        }
    }

    pub fn backspace(&mut self) -> bool {
        self.digits.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.digits.clear();
    }

    pub fn take_raw(&mut self, engine: &mut Engine) -> String {
        engine.clear();
        std::mem::take(&mut self.digits)
    }

    pub fn query(&self, engine: &mut Engine) -> CandidateList {
        if self.digits.is_empty() {
            return CandidateList::default();
        }
        let n = self.digits.len();
        let mut paths = vec![Vec::<Path>::new(); n + 1];
        paths[0].push(Path::default());
        let total = engine.dictionary.total_frequency().max(1) as f64;
        for start in 0..n {
            let previous = paths[start].clone();
            if previous.is_empty() {
                continue;
            }
            for (end, destination) in paths
                .iter_mut()
                .enumerate()
                .take((start + 32).min(n) + 1)
                .skip(start + 1)
            {
                let code = &self.digits[start..end];
                for word in self.index.exact(code) {
                    for path in &previous {
                        let probability = engine
                            .language_model
                            .log_prob(path.previous.as_deref(), &word.text)
                            .unwrap_or_else(|| ((word.frequency.max(1) as f64) / total).ln());
                        let bonus = crate::ranking::weight_bonus(engine.learner.weight(&word.text));
                        let mut next = path.clone();
                        next.text.push_str(&word.text);
                        next.syllables.extend(word.syllables.iter().cloned());
                        next.previous = Some(word.text.clone());
                        next.score += probability + bonus;
                        next.words += 1;
                        destination.push(next);
                    }
                }
                destination.sort_by(|a, b| b.score.total_cmp(&a.score));
                destination.truncate(8);
            }
        }
        let mut items: Vec<Candidate> = paths[n]
            .iter()
            .map(|p| Candidate {
                text: p.text.clone(),
                kind: if p.words > 1 {
                    CandidateKind::Sentence
                } else {
                    CandidateKind::Chinese
                },
                syllables: p.syllables.clone(),
                reading: None,
                translation: None,
                aux_code: None,
            })
            .collect();
        let mut completions = self.index.prefix(&self.digits);
        completions.sort_by(|a, b| {
            let score = |word: &entry::Entry| {
                (word.frequency.max(1) as f64).ln()
                    + crate::ranking::weight_bonus(engine.learner.weight(&word.text))
            };
            (b.code == self.digits)
                .cmp(&(a.code == self.digits))
                .then_with(|| score(b).total_cmp(&score(a)))
        });
        items.extend(completions.iter().map(|entry| entry.candidate()));
        // 完整整句之外仍提供前缀词，选词后余下的数字继续组句。
        for end in (1..n).rev() {
            items.extend(
                self.index
                    .exact(&self.digits[..end])
                    .iter()
                    .take(3)
                    .map(|e| e.candidate()),
            );
        }
        let mut seen = HashSet::new();
        items.retain(|item| seen.insert(item.text.clone()));
        items.truncate(64);
        if let Some(first) = items.first() {
            Self::prepare(engine, first);
        } else {
            engine.clear();
        }
        CandidateList { items }
    }

    fn prepare(engine: &mut Engine, candidate: &Candidate) {
        let pinyin = candidate.syllables.join("");
        if engine.composition().text() != pinyin {
            engine.clear();
            for letter in pinyin.chars() {
                engine.push(letter);
            }
        }
    }

    pub fn commit(
        &mut self,
        engine: &mut Engine,
        candidate: &Candidate,
        translation: bool,
    ) -> Option<String> {
        if translation && candidate.translation.is_none() {
            return None;
        }
        let count = encode(&candidate.syllables.join(" "))
            .len()
            .min(self.digits.len());
        Self::prepare(engine, candidate);
        let result = if translation {
            engine.commit_translation(candidate, 0)
        } else {
            Some(engine.commit(candidate))
        };
        if result.is_some() {
            self.digits.drain(..count);
        }
        result
    }
}
