//! Android 的引擎装配，读取随包产品词库与应用私有目录中的学习数据。

use std::path::Path;

use qingjian_core::{Engine, Language};
use qingjian_dictionary::{Dictionary, WordList};
use qingjian_learning::FrequencyLearner;
use qingjian_lm::BigramModel;
use qingjian_translate::Glossary;

pub(crate) fn assemble(data: &Path, user: &Path) -> Result<Engine, String> {
    std::fs::create_dir_all(user).map_err(|error| error.to_string())?;
    let dictionary = Dictionary::from_path(data.join("dict.qj"))
        .map_err(|error| format!("词库加载失败：{error}"))?;
    let model = BigramModel::from_path(&data.join("lm.qj"))
        .map_err(|error| format!("组句数据加载失败：{error}"))?;
    let learner = FrequencyLearner::from_path(user.join("user.tsv"))
        .map_err(|error| format!("学习数据读取失败：{error}"))?;
    let mut engine = Engine::new(dictionary)
        .with_language_model(Box::new(model))
        .with_learner(Box::new(learner));
    if data.join("english.tsv").is_file() {
        engine = engine.with_english(
            WordList::from_path(data.join("english.tsv")).map_err(|error| error.to_string())?,
        );
    }
    if data.join("glossary-zh.qj").is_file() {
        engine = engine.with_english_translator(Box::new(
            Glossary::from_path(Language::Chinese, data.join("glossary-zh.qj"))
                .map_err(|error| error.to_string())?,
        ));
    }
    Ok(engine)
}

pub(crate) fn glossary(data: &Path, language: &str) -> Result<Glossary, String> {
    let language = match language {
        "en" => Language::English,
        "ja" => Language::Japanese,
        "es" => Language::Spanish,
        _ => return Err("学习语言需为英语、日语或西班牙语".to_owned()),
    };
    Glossary::from_path(
        language,
        data.join(format!("glossary-{}.qj", language.code())),
    )
    .map_err(|error| format!("释义表加载失败：{error}"))
}
