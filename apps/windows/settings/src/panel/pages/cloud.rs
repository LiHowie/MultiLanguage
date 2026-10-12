//! 「云服务」页：本地整句模型开关（`[model]`）、`[predict]` 各项与「测试连接」（后台线程跑）。

use qingjian_predict::{ConnectionTest, PredictConfig, TranslationStyle};
use windows_reactor::*;

use crate::panel::cloud_status::CloudStatus;
use crate::panel::controls::{field, labeled, note, page};
use crate::panel::{Message, Settings};

/// 后台跑一次连通性测试，轮询到有结果或被取消。
pub(crate) fn run_test(
    config: &PredictConfig,
    cancel: &CancellationToken,
) -> Result<String, String> {
    if config.membership_service {
        let value = qingjian_predict::MembershipClient::new(config)?.block("/me", None)?;
        return Ok(format!(
            "连接成功：{}",
            value["user"]["email"].as_str().unwrap_or("魔多账号")
        ));
    }
    let test = ConnectionTest::start(config).map_err(|error| error.to_string())?;
    loop {
        if cancel.is_cancelled() {
            return Err("已取消".to_owned());
        }
        if let Some(result) = test.poll() {
            return result
                .map(|report| {
                    format!(
                        "连接成功：模型 {}，耗时 {} ms",
                        report.model,
                        report.elapsed.as_millis()
                    )
                })
                .map_err(|error| error.to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let p = &settings.config.predict;
    if p.membership_service {
        return page("整句翻译", StackPanel::new().spacing(16.0).children([
            field("会员整句翻译", "停顿后翻译当前整句候选；基础打字和本地单词译词继续免费。先在会员与语音老师页登录并同意云端处理。",
                ToggleSwitch::new().is_on(p.sentence_translation).on_toggled(context.callback(Message::SentenceTranslation))),
            field("翻译风格", "口语化适合聊天；书面化适合邮件和文章。",
                ComboBox::new().items_source(["口语化", "书面化"]).selected_index(TranslationStyle::ALL.iter().position(|style| *style == p.translation_style))
                    .on_selection_changed(context.callback(Message::TranslationStyle))),
            field("本地整句模型", "在本机调整中文候选排序，不消耗会员额度。",
                ToggleSwitch::new().is_on(settings.config.model.enabled).on_toggled(context.callback(Message::LocalModel))),
            note("普通选词输出中文，Ctrl + 数字输出译文。会员到期或网络异常时仍可正常输入中文。"),
        ]));
    }
    let status = match &settings.cloud_status {
        CloudStatus::Idle => String::new(),
        CloudStatus::Testing => "测试中…".to_owned(),
        CloudStatus::Ok(message) => message.clone(),
        CloudStatus::Failed(message) => format!("失败：{message}"),
    };
    let rows = [
        field(
            "本地整句模型",
            "随包的小模型在本机给整句候选重新排序，全程离线；停键后几十毫秒生效。关掉只用词库统计。",
            ToggleSwitch::new()
                .is_on(settings.config.model.enabled)
                .on_toggled(context.callback(Message::LocalModel)),
        ),
        field(
            "整句 AI 翻译",
            "停顿后把当前整句候选发给下面的 AI，译文显示在候选下方。目标跟随通用页的学习语言；可单独开启，无需云联想。普通选词输出中文，Ctrl + 数字输出译文。",
            ToggleSwitch::new()
                .is_on(p.sentence_translation)
                .on_toggled(context.callback(Message::SentenceTranslation)),
        ),
        field(
            "翻译风格",
            "口语化适合聊天和口头表达；书面化适合邮件和文章。也用于 Ctrl + Alt + T 翻译选中文字。",
            ComboBox::new()
                .items_source(["口语化", "书面化"])
                .selected_index(
                    TranslationStyle::ALL
                        .iter()
                        .position(|style| *style == p.translation_style),
                )
                .on_selection_changed(context.callback(Message::TranslationStyle)),
        ),
        field(
            "启用云联想",
            "开启后组句时会把光标附近的几十个字发给下面的服务，让模型补全整句、联想下文；密钥框里的内容不发送。",
            ToggleSwitch::new()
                .is_on(p.enabled)
                .on_toggled(context.callback(Message::CloudEnabled)),
        ),
        field(
            "云端词格数",
            "云端词到了只补进第一页末尾这几格，前面的本地候选不动；没到就什么都不变。0 = 只要整句补全。",
            NumberBox::new()
                .minimum(0.0)
                .maximum(9.0)
                .value(p.slots as f64)
                .on_value_changed(context.callback(Message::CloudSlots)),
        ),
        field(
            "整句补全",
            "preedit 右侧给出整句补全，按 Tab 采用。",
            ToggleSwitch::new()
                .is_on(p.sentence)
                .on_toggled(context.callback(Message::CloudSentence)),
        ),
        field(
            "接口地址",
            "",
            TextBox::new()
                .text(p.base_url.clone())
                .on_text_changed(context.callback(Message::CloudBaseUrl)),
        ),
        field(
            "模型",
            "",
            TextBox::new()
                .text(p.model.clone())
                .on_text_changed(context.callback(Message::CloudModel)),
        ),
        field(
            "API 密钥",
            "只保存在这台电脑上，不会随配置文件导出，也不显示已填的值。留空则读环境变量 QINGJIAN_API_KEY。",
            PasswordBox::new()
                .password(p.api_key.clone().unwrap_or_default())
                .placeholder_text("留空则读环境变量 QINGJIAN_API_KEY")
                .on_password_changed(context.callback(Message::CloudApiKey)),
        ),
        labeled(
            "",
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(12.0)
                .children((
                    Button::new()
                        .on_click(context.message(Message::TestConnection))
                        .content("测试连接"),
                    TextBlock::new().text(status),
                )),
        ),
        note(
            "用上面填的地址、模型、密钥发一条最小请求。走不通时先查这里；Server 进程看不到终端里的代理变量。",
        ),
    ];
    page("云服务", StackPanel::new().spacing(16.0).children(rows))
}
