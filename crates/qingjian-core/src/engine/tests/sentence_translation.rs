//! 整句译文的代次隔离、语言与隐私边界、缓存和提交。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::{FixedTranslator, engine};
use crate::{
    Candidate, CandidateKind, CandidateList, Language, NoTranslator, SentenceTranslationRequest,
    SentenceTranslationResult, SentenceTranslator,
};

#[derive(Default)]
struct Mock {
    requests: Vec<SentenceTranslationRequest>,

    results: VecDeque<SentenceTranslationResult>,
}

impl SentenceTranslator for Arc<Mutex<Mock>> {
    fn submit(&mut self, request: SentenceTranslationRequest) {
        self.lock().unwrap().requests.push(request);
    }
    fn cancel(&mut self) {}
    fn poll(&mut self) -> Option<SentenceTranslationResult> {
        self.lock().unwrap().results.pop_front()
    }
}

fn sentence(text: &str) -> Candidate {
    Candidate {
        text: text.to_owned(),
        kind: CandidateKind::Sentence,
        syllables: ["wo", "xiang", "kai", "fa"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        reading: None,
        translation: None,
        aux_code: None,
    }
}

fn setup() -> (crate::Engine, Arc<Mutex<Mock>>) {
    let mock = Arc::new(Mutex::new(Mock::default()));
    let mut engine = engine().with_translator(Box::new(FixedTranslator));
    engine.set_sentence_translator(Some(Box::new(mock.clone())));
    (engine, mock)
}

fn reply(mock: &Arc<Mutex<Mock>>, sequence: u64, text: Option<&str>) {
    mock.lock()
        .unwrap()
        .results
        .push_back(SentenceTranslationResult {
            sequence,
            text: text.map(str::to_owned),
        });
}

#[test]
fn requests_are_deduplicated_and_old_sentence_results_are_discarded() {
    let (mut engine, mock) = setup();
    let old = sentence("我想开发软件");
    engine.request_sentence_translation(std::slice::from_ref(&old));
    engine.request_sentence_translation(&[old]);
    let current = sentence("我想学习英语");
    engine.request_sentence_translation(std::slice::from_ref(&current));
    let requests = mock.lock().unwrap().requests.clone();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[1].language, Language::English);
    reply(&mock, requests[0].sequence, Some("stale"));
    reply(
        &mock,
        requests[1].sequence,
        Some("I want to learn English."),
    );
    assert!(engine.poll_sentence_translation());
    let mut list = CandidateList {
        items: vec![sentence("我想开发软件"), current.clone()],
    };
    engine.annotate(&mut list);
    assert!(list.items[0].translation.is_none());
    assert_eq!(
        list.items[1].translation.as_ref().unwrap().senses()[0].text,
        "I want to learn English."
    );
    assert!(!list.items[1].translation.as_ref().unwrap().senses()[0].fresh);
    engine.request_sentence_translation(&[current]);
    assert_eq!(mock.lock().unwrap().requests.len(), 2);
}

#[test]
fn privacy_language_change_clear_and_reconfiguration_invalidate_results() {
    for boundary in ["private", "language", "clear", "config"] {
        let (mut engine, mock) = setup();
        let source = sentence("我想开发软件");
        engine.request_sentence_translation(std::slice::from_ref(&source));
        let seq = mock.lock().unwrap().requests[0].sequence;
        match boundary {
            "private" => engine.set_private(true),
            "language" => engine.set_translator(Box::new(NoTranslator)),
            "clear" => engine.clear(),
            _ => engine.set_sentence_translator(Some(Box::new(mock.clone()))),
        }
        reply(&mock, seq, Some("stale"));
        assert!(!engine.poll_sentence_translation(), "{boundary}");
        let mut list = CandidateList {
            items: vec![source],
        };
        engine.annotate(&mut list);
        assert!(list.items[0].translation.is_none(), "{boundary}");
    }
}

#[test]
fn disabled_learning_language_and_private_input_never_submit() {
    let (mut engine, mock) = setup();
    engine.set_private(true);
    engine.request_sentence_translation(&[sentence("我想开发软件")]);
    engine.set_private(false);
    engine.set_translator(Box::new(NoTranslator));
    engine.request_sentence_translation(&[sentence("我想开发软件")]);
    engine.set_translator(Box::new(FixedTranslator));
    engine.request_sentence_translation(&[sentence("我")]);
    let mut word = sentence("我想开发软件");
    word.kind = CandidateKind::Chinese;
    engine.request_sentence_translation(&[word]);
    engine.request_sentence_translation(&[sentence(&"我".repeat(501))]);
    assert!(mock.lock().unwrap().requests.is_empty());
}

#[test]
fn failed_request_can_be_retried_and_cached_translation_can_be_committed() {
    let (mut engine, mock) = setup();
    engine.set_input("woxiangkaifa");
    let candidate = sentence("我想开发");
    engine.request_sentence_translation(std::slice::from_ref(&candidate));
    let sequence = mock.lock().unwrap().requests[0].sequence;
    reply(&mock, sequence, None);
    assert!(!engine.poll_sentence_translation());
    engine.request_sentence_translation(std::slice::from_ref(&candidate));
    let sequence = mock.lock().unwrap().requests[1].sequence;
    reply(&mock, sequence, Some("I'd like to develop it."));
    assert!(engine.poll_sentence_translation());
    let mut list = CandidateList {
        items: vec![candidate],
    };
    engine.annotate(&mut list);
    assert_eq!(
        engine.commit_translation(&list.items[0], 0).as_deref(),
        Some("I'd like to develop it.")
    );
    assert!(engine.composition().is_empty());
}

#[test]
fn cache_is_bounded_and_reconfiguring_removes_previous_style() {
    let (mut engine, mock) = setup();
    for index in 0..65 {
        let candidate = sentence(&format!("测试句子{index}"));
        engine.request_sentence_translation(&[candidate]);
        let sequence = mock.lock().unwrap().requests.last().unwrap().sequence;
        reply(&mock, sequence, Some("Translated sentence."));
        assert!(engine.poll_sentence_translation());
    }
    let mut list = CandidateList {
        items: vec![sentence("测试句子0"), sentence("测试句子64")],
    };
    engine.annotate(&mut list);
    assert!(list.items[0].translation.is_none());
    assert!(list.items[1].translation.is_some());
    engine.set_sentence_translator(Some(Box::new(mock)));
    engine.annotate(&mut list);
    assert!(
        list.items
            .iter()
            .all(|candidate| candidate.translation.is_none())
    );
}
