//! Android 会话桥的中文提交、译文提交、退格与隐私边界。

use std::path::PathBuf;

use qingjian_core::{CandidateList, Engine, Language};
use qingjian_dictionary::Dictionary;
use qingjian_translate::Glossary;

use super::Session;

fn session() -> Session {
    let dictionary = Dictionary::parse("你好\tni hao\t1000\n你\tni\t500\n好\thao\t400\n").unwrap();
    let glossary = Glossary::parse(Language::English, "你好\thello\n你\tyou\n好\tgood\n").unwrap();
    Session {
        engine: Engine::new(dictionary).with_translator(Box::new(glossary)),
        data: PathBuf::new(),
        candidates: CandidateList::default(),
        translation_enabled: false,
        configuration_error: None,
        keypad: None,
    }
}

#[test]
fn committing_chinese_and_translation_both_consume_composition() {
    let mut session = session();
    let frame = session.command("push", "nihao");
    assert_eq!(frame.input, "nihao");
    assert_eq!(frame.candidates[0].text, "你好");
    assert_eq!(frame.candidates[0].translation.as_deref(), Some("hello"));
    let frame = session.command("translate", "0");
    assert_eq!(frame.commit.as_deref(), Some("hello"));
    assert!(frame.input.is_empty());
    assert!(frame.candidates.is_empty());
    session.command("push", "nihao");
    assert_eq!(session.command("space", "").commit.as_deref(), Some("你好"));
}

#[test]
fn privacy_reset_cancels_composition_and_never_reuses_a_translation() {
    let mut session = session();
    session.translation_enabled = true;
    session.command("push", "nihao");
    let frame = session.command("reset", "private");
    assert!(frame.private);
    assert!(!frame.translation_enabled);
    assert!(frame.input.is_empty());
    assert!(frame.candidates.is_empty());
    assert!(session.command("translate", "0").commit.is_none());
}

#[test]
fn backspace_deletes_editor_text_only_after_composition_is_empty() {
    let mut session = session();
    session.command("push", "ni");
    let frame = session.command("backspace", "");
    assert!(!frame.delete);
    assert_eq!(frame.input, "n");
    session.command("backspace", "");
    assert!(session.command("backspace", "").delete);
}

#[test]
fn failed_or_missing_translation_preserves_chinese_composition() {
    let mut session = session();
    session.command("push", "hao");
    session
        .engine
        .set_translator(Box::new(qingjian_core::NoTranslator));
    session.command("poll", "");
    let frame = session.command("translate", "0");
    assert!(frame.commit.is_none());
    assert!(frame.error.is_some());
    assert_eq!(frame.input, "hao");
}

#[test]
fn punctuation_commits_candidate_before_punctuation() {
    let mut session = session();
    session.command("push", "nihao");
    let frame = session.command("text", ",");
    assert_eq!(frame.commit.as_deref(), Some("你好，"));
    assert!(frame.input.is_empty());
}
