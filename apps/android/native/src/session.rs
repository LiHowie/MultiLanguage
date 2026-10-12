//! Android 输入会话；把触摸事件交给 Engine，再序列化当前候选。

use std::path::{Path, PathBuf};

use qingjian_core::{Candidate, CandidateList, Engine, Keypad};
use qingjian_predict::CloudSentenceTranslator;

use crate::assembly;
use crate::frame::Frame;
use crate::row::Row;
use crate::settings::Settings;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub(crate) struct Session {
    pub engine: Engine,

    data: PathBuf,

    candidates: CandidateList,

    translation_enabled: bool,

    configuration_error: Option<String>,

    keypad: Option<Keypad>,
}

impl Session {
    pub fn new(data: &Path, user: &Path, settings: Settings) -> Result<Self, String> {
        let engine = assembly::assemble(data, user)?;
        let mut session = Self {
            engine,
            data: data.to_owned(),
            candidates: CandidateList::default(),
            translation_enabled: false,
            configuration_error: None,
            keypad: None,
        };
        session.configure(settings)?;
        Ok(session)
    }

    pub fn configure(&mut self, settings: Settings) -> Result<(), String> {
        let glossary = assembly::glossary(&self.data, &settings.learning_language)?;
        self.configuration_error = None;
        let provider = if settings.predict.sentence_translation {
            match CloudSentenceTranslator::new(&settings.predict) {
                Ok(provider) => {
                    Some(Box::new(provider) as Box<dyn qingjian_core::SentenceTranslator>)
                }
                Err(error) => {
                    self.configuration_error = Some(format!("整句翻译配置错误：{error}"));
                    None
                }
            }
        } else {
            None
        };
        self.engine.clear();
        if let Some(keypad) = &mut self.keypad {
            keypad.clear();
        }
        self.engine.set_translator(Box::new(glossary));
        self.translation_enabled = provider.is_some();
        self.engine.set_sentence_translator(provider);
        self.candidates = CandidateList::default();
        Ok(())
    }

    pub fn command(&mut self, action: &str, value: &str) -> Frame {
        let mut frame = Frame::default();
        match action {
            "push9" => {
                let keypad = self.keypad.get_or_insert_with(|| Keypad::new(&self.engine));
                for digit in value.chars() {
                    keypad.push(digit);
                }
            }
            "mode" => {
                self.engine.clear();
                if let Some(keypad) = &mut self.keypad {
                    keypad.clear();
                }
                self.engine.set_english_mode(value == "en");
                self.engine.set_full_width_punctuation(value != "en");
            }
            "push" => {
                for character in value.chars().take(128) {
                    if self.engine.composition().text().len() < 512 {
                        self.engine.push(character);
                    }
                }
            }
            "backspace" => {
                if let Some(keypad) = &mut self.keypad
                    && !keypad.input().is_empty()
                {
                    keypad.backspace();
                    if keypad.input().is_empty() {
                        self.engine.clear();
                    }
                } else {
                    frame.delete = !self.engine.backspace();
                }
            }
            "reset" => {
                self.engine.clear();
                if let Some(keypad) = &mut self.keypad {
                    keypad.clear();
                }
                self.engine.set_private(value == "private");
            }
            "clear" => {
                self.engine.clear();
                if let Some(keypad) = &mut self.keypad {
                    keypad.clear();
                }
            }
            "poll" => {
                self.engine.poll_sentence_translation();
            }
            "raw" => {
                frame.commit = Some(
                    if let Some(keypad) = &mut self.keypad
                        && !keypad.input().is_empty()
                    {
                        keypad.take_raw(&mut self.engine)
                    } else {
                        self.engine.take_raw()
                    },
                )
            }
            "select" | "translate" => {
                let candidate = value
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| self.candidates.items.get(index))
                    .cloned();
                if let Some(candidate) = candidate {
                    frame.commit = self.commit(&candidate, action == "translate");
                    if frame.commit.is_none() {
                        frame.error = Some("译文尚未就绪，请稍候或检查 AI 连接".to_owned());
                    }
                }
            }
            "space" => {
                if let Some(candidate) = self.candidates.items.first().cloned() {
                    frame.commit = self.commit(&candidate, false);
                } else if self.engine.composition().is_empty() {
                    frame.commit = Some(" ".to_owned());
                } else {
                    frame.commit = Some(self.engine.take_raw());
                }
            }
            "text" => {
                let mut commit = String::new();
                if let Some(candidate) = self.candidates.items.first().cloned() {
                    commit.push_str(&self.commit(&candidate, false).unwrap_or_default());
                }
                if !self.engine.composition().is_empty() {
                    commit.push_str(&self.engine.take_raw());
                }
                for character in value.chars() {
                    if let Some(punctuation) = self.engine.punctuate(character) {
                        commit.push_str(punctuation);
                    } else {
                        commit.push(character);
                    }
                }
                frame.commit = Some(commit);
            }
            _ => frame.error = Some("不支持的键盘操作".to_owned()),
        }
        self.refresh(&mut frame, action != "poll");
        if frame.commit.is_some() {
            self.engine.flush_learning();
        }
        frame
    }

    fn refresh(&mut self, frame: &mut Frame, query: bool) {
        frame.input = self
            .keypad
            .as_ref()
            .map(Keypad::input)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.engine.composition().text())
            .to_owned();
        frame.private = self.engine.is_private();
        frame.translation_enabled = self.translation_enabled && !frame.private;
        if frame.error.is_none() {
            frame.error = self.configuration_error.clone();
        }
        if query {
            self.candidates = if let Some(keypad) = &self.keypad
                && !keypad.input().is_empty()
            {
                keypad.query(&mut self.engine)
            } else if self.engine.composition().is_empty() {
                CandidateList::default()
            } else {
                self.engine
                    .query()
                    .map(|query| query.candidates)
                    .unwrap_or_default()
            };
            self.engine
                .request_sentence_translation(&self.candidates.items);
        }
        self.engine.annotate(&mut self.candidates);
        frame.candidates = self
            .candidates
            .items
            .iter()
            .take(64)
            .map(Row::from)
            .collect();
    }

    fn commit(&mut self, candidate: &Candidate, translation: bool) -> Option<String> {
        if let Some(keypad) = &mut self.keypad
            && !keypad.input().is_empty()
        {
            keypad.commit(&mut self.engine, candidate, translation)
        } else if translation {
            self.engine.commit_translation(candidate, 0)
        } else {
            Some(self.engine.commit(candidate))
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.engine.clear();
        self.engine.flush_learning();
    }
}
