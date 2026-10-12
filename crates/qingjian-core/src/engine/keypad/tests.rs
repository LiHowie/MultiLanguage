//! 九键消歧、整句与选词后继续输入的回归测试。
use super::{Keypad, index::encode};
use crate::{CandidateKind, Engine};
use qingjian_dictionary::Dictionary;

fn engine() -> Engine {
    Engine::new(Dictionary::parse("你好\tni hao\t1000\n你\tni\t100\n好\thao\t100\n我\two\t100\n学习\txue xi\t200\n外语\twai yu\t300\n米\tmi\t20\n").unwrap())
}

#[test]
fn keypad_uses_real_pinyin_and_leaves_remaining_digits() {
    let mut engine = engine();
    let mut keypad = Keypad::new(&engine);
    for c in "64426".chars() {
        keypad.push(c);
    }
    let list = keypad.query(&mut engine);
    assert_eq!(list.items[0].text, "你好");
    let first = list.items.iter().find(|c| c.text == "你").unwrap();
    assert_eq!(
        keypad.commit(&mut engine, first, false).as_deref(),
        Some("你")
    );
    assert_eq!(keypad.input(), "426");
    let list = keypad.query(&mut engine);
    assert_eq!(list.items[0].text, "好");
    assert_eq!(
        keypad.commit(&mut engine, &list.items[0], false).as_deref(),
        Some("好")
    );
    assert!(keypad.input().is_empty());
}

#[test]
fn keypad_composes_sentences_outside_the_word_dictionary() {
    let mut engine = engine();
    let mut keypad = Keypad::new(&engine);
    for c in encode("wo xue xi wai yu").chars() {
        keypad.push(c);
    }
    let list = keypad.query(&mut engine);
    assert_eq!(list.items[0].text, "我学习外语");
    assert_eq!(list.items[0].kind, CandidateKind::Sentence);
    assert_eq!(
        keypad.commit(&mut engine, &list.items[0], false).as_deref(),
        Some("我学习外语")
    );
    assert!(keypad.input().is_empty());
    assert!(engine.composition().is_empty());
}

#[test]
fn keypad_delete_and_missing_translation_do_not_drop_input() {
    let mut engine = engine();
    let mut keypad = Keypad::new(&engine);
    for c in "64".chars() {
        keypad.push(c);
    }
    let list = keypad.query(&mut engine);
    assert!(keypad.commit(&mut engine, &list.items[0], true).is_none());
    assert_eq!(keypad.input(), "64");
    assert!(keypad.backspace());
    assert_eq!(keypad.input(), "6");
    assert_eq!(keypad.take_raw(&mut engine), "6");
    assert!(engine.composition().is_empty());
}
