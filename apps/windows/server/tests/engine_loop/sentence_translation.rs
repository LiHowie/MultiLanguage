//! 整句翻译的 Windows 协议闭环：不依赖云联想，异步刷新与译文上屏。

use std::collections::VecDeque;

use crate::support::{
    Arc, ClientMessage, KeyEvent, KeyModifiers, Mutex, SESSION, ServerMessage, digit, key_result,
    press, router, type_letters,
};
use qingjian_core::{SentenceTranslationRequest, SentenceTranslationResult, SentenceTranslator};

#[derive(Default)]
struct EchoTranslator {
    requests: Arc<Mutex<Vec<SentenceTranslationRequest>>>,

    results: VecDeque<SentenceTranslationResult>,
}

impl SentenceTranslator for EchoTranslator {
    fn submit(&mut self, request: SentenceTranslationRequest) {
        self.results.push_back(SentenceTranslationResult {
            sequence: request.sequence,
            text: Some(format!("EN: {}", request.text)),
        });
        self.requests.lock().unwrap().push(request);
    }

    fn cancel(&mut self) {}

    fn poll(&mut self) -> Option<SentenceTranslationResult> {
        self.results.pop_front()
    }
}

#[test]
fn polling_displays_translation_and_ctrl_digit_commits_it_with_prediction_off() {
    let mut router = router();
    let mock = Arc::new(Mutex::new(Vec::new()));
    assert!(!router.engine_mut().prediction_enabled());
    router
        .engine_mut()
        .set_sentence_translator(Some(Box::new(EchoTranslator {
            requests: mock.clone(),
            ..Default::default()
        })));
    let (_, _, before) = type_letters(&mut router, "nihaowoshikaifazhe");
    let source = before.candidates.items[0].text.clone();
    assert!(before.candidates.items[0].translation.is_none());
    let Some(ServerMessage::Update { frame, .. }) =
        router.handle(ClientMessage::Poll { session: SESSION })
    else {
        panic!("轮询应返回一帧");
    };
    let expected = format!("EN: {source}");
    assert_eq!(
        frame.candidates.items[0]
            .translation
            .as_ref()
            .unwrap()
            .senses()[0]
            .text,
        expected
    );
    let (_, commit, frame) = key_result(router.handle(ClientMessage::Key {
        session: SESSION,
        event: KeyEvent::new(
            '1' as u32,
            Some('1'),
            KeyModifiers {
                ctrl: true,
                ..Default::default()
            },
        ),
    }));
    assert_eq!(commit, Some(expected));
    assert!(frame.preedit.is_empty());
    assert!(!mock.lock().unwrap().is_empty());
}

#[test]
fn normal_selection_still_commits_chinese_after_translation_arrives() {
    let mut router = router();
    router
        .engine_mut()
        .set_sentence_translator(Some(Box::new(EchoTranslator::default())));
    let (_, _, before) = type_letters(&mut router, "nihaowoshikaifazhe");
    let source = before.candidates.items[0].text.clone();
    router.handle(ClientMessage::Poll { session: SESSION });
    let (_, commit, _) = press(&mut router, digit(1));
    assert_eq!(commit, Some(source));
}
