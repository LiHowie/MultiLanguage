//! 九键整句的有界搜索路径。
#[derive(Clone, Default)]
pub(super) struct Path {
    pub text: String,

    pub syllables: Vec<String>,

    pub previous: Option<String>,

    pub score: f64,

    pub words: usize,
}
