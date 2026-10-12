//! 整句与选区翻译的表达风格，跟随用户配置。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStyle {
    #[default]
    Conversational,

    Formal,
}

impl TranslationStyle {
    pub const ALL: [Self; 2] = [Self::Conversational, Self::Formal];

    pub fn code(self) -> &'static str {
        match self {
            Self::Conversational => "conversational",
            Self::Formal => "formal",
        }
    }

    pub(crate) fn instruction(self) -> &'static str {
        match self {
            Self::Conversational => {
                "翻译风格：口语化。使用日常交流中自然、简洁的说法；适合聊天和口头表达，避免生硬直译。保持原意、事实和礼貌程度，不添加内容。"
            }
            Self::Formal => {
                "翻译风格：书面化。使用规范、清晰的书面表达；适合邮件、文章和正式交流，避免俚语与随意缩写。保持原意、事实和礼貌程度，不添加内容。"
            }
        }
    }
}
