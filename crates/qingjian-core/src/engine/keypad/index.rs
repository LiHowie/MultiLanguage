//! 数字码二分索引；短前缀预选避免每次按键遍历整个词库。
use super::super::Engine;
use super::entry::Entry;
use std::collections::HashMap;

pub(super) struct Index {
    entries: Vec<Entry>,

    short: HashMap<String, Vec<usize>>,
}

pub(super) fn encode(pinyin: &str) -> String {
    pinyin
        .bytes()
        .filter_map(|b| {
            Some(match b {
                b'a'..=b'c' => '2',
                b'd'..=b'f' => '3',
                b'g'..=b'i' => '4',
                b'j'..=b'l' => '5',
                b'm'..=b'o' => '6',
                b'p'..=b's' => '7',
                b't'..=b'v' => '8',
                b'w'..=b'z' => '9',
                _ => return None,
            })
        })
        .collect()
}

impl Index {
    pub fn new(engine: &Engine) -> Self {
        let mut entries: Vec<Entry> = engine
            .all_dictionaries()
            .into_iter()
            .flat_map(|d| d.entries())
            .map(|word| Entry {
                code: encode(word.pinyin),
                text: word.text.to_owned(),
                syllables: word.syllables().map(str::to_owned).collect(),
                frequency: word.frequency,
            })
            .filter(|word| !word.code.is_empty())
            .collect();
        entries.sort_by(|a, b| {
            a.code
                .cmp(&b.code)
                .then_with(|| b.frequency.cmp(&a.frequency))
        });
        let mut short: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, entry) in entries.iter().enumerate() {
            for length in 1..=entry.code.len().min(3) {
                short
                    .entry(entry.code[..length].to_owned())
                    .or_default()
                    .push(index);
            }
        }
        for items in short.values_mut() {
            items.sort_unstable_by_key(|i| std::cmp::Reverse(entries[*i].frequency));
            items.truncate(32);
        }
        Self { entries, short }
    }

    pub fn exact(&self, code: &str) -> &[Entry] {
        let start = self.entries.partition_point(|e| e.code.as_str() < code);
        let end = self.entries.partition_point(|e| e.code.as_str() <= code);
        &self.entries[start..end.min(start + 24)]
    }

    pub fn prefix(&self, code: &str) -> Vec<&Entry> {
        if let Some(items) = self.short.get(code) {
            return items.iter().map(|i| &self.entries[*i]).collect();
        }
        let start = self.entries.partition_point(|e| e.code.as_str() < code);
        let mut items: Vec<&Entry> = self.entries[start..]
            .iter()
            .take_while(|e| e.code.starts_with(code))
            .collect();
        items.sort_unstable_by_key(|e| std::cmp::Reverse(e.frequency));
        items.truncate(32);
        items
    }
}
