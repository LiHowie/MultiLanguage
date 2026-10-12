//! 当前整句候选的在线翻译，不参与候选排序。

mod prompt;
mod translator;
mod worker;

#[cfg(test)]
mod tests;

pub use translator::CloudSentenceTranslator;
