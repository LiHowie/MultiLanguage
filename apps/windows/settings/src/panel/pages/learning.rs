//! 魔多会员、跨设备学习记录和语音老师页面。
use crate::panel::controls::{field, note, page};
use crate::panel::learning::LearningMessage;
use crate::panel::{Message, Settings};
use windows_reactor::*;

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let s = &settings.learning;
    let callback = |context: &mut ViewContext<Settings>, f: fn(String) -> LearningMessage| {
        context.callback(move |v| Message::Membership(f(v)))
    };
    let speech = &s.speech;
    let result = if speech.is_null() {
        "录音结果显示在这里；外语学习模式会保留正确句子，供下一次跟读。".to_owned()
    } else if let Some(text) = speech["output"].as_str() {
        text.to_owned()
    } else {
        format!(
            "原句：{}\n含义：{}\n正确表达：{}\n{}\n{}",
            speech["transcript"].as_str().unwrap_or(""),
            speech["motherMeaning"].as_str().unwrap_or(""),
            speech["correctedText"].as_str().unwrap_or(""),
            speech["reading"].as_str().unwrap_or(""),
            speech["explanation"].as_str().unwrap_or("")
        )
    };

    page("魔多学习会员",StackPanel::new().spacing(14.0).children([
        note("基础输入与本地词汇翻译免费。整句翻译和语音老师：20 元/月，1,000 次翻译和120分钟语音。开发版暂不开放付款，测试会员由运营人员开通。"),
        field("服务地址","包括 /v1；远程服务需使用 HTTPS",TextBox::new().text(s.url.clone()).on_text_changed(callback(context,LearningMessage::Url))),
        field("邮箱","",TextBox::new().text(s.email.clone()).on_text_changed(callback(context,LearningMessage::Email))),
        field("密码","至少10位，仅用于本次登录",PasswordBox::new().password(s.password.clone()).on_password_changed(callback(context,LearningMessage::Password))),
        buttons(context,[("登录",LearningMessage::Login(false)),("注册",LearningMessage::Login(true)),("刷新会员",LearningMessage::Refresh)]),
        buttons(context,[("退出账号",LearningMessage::Logout)]),
        field("云端学习","允许发送本次待翻译文字和主动录音；不会同步此设备的同意选择",ToggleSwitch::new().is_on(settings.config.predict.cloud_consent).on_toggled(context.callback(|v|Message::Membership(LearningMessage::Consent(v))))),
        field("语音模式","母语输入输出识别原文；学习模式增加中文含义与表达纠错",ComboBox::new().items_source(["母语输入","外语学习"]).selected_index(Some(usize::from(s.learning_mode))).on_selection_changed(context.callback(|v|Message::Membership(LearningMessage::Mode(v))))),
        buttons(context,[("开始录音 / 再次跟读",LearningMessage::Record),("停止录音",LearningMessage::Stop)]),
        buttons(context,[("取消录音",LearningMessage::Cancel)]),
        note(&result),
        buttons(context,[("准备插入（回到输入框按 F8）",LearningMessage::Prepare),("收藏纠正句子",LearningMessage::SaveCard)]),
        buttons(context,[("拉取学习记录",LearningMessage::Pull),("同步本机学习设置",LearningMessage::Sync)]),
        note(&s.notice),
        note("AI 提供表达建议，不进行发音评分。录音最长30秒，不落盘原始音频。"),
    ]))
}

fn buttons<const N: usize>(
    context: &mut ViewContext<Settings>,
    items: [(&str, LearningMessage); N],
) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(10.0)
        .children(items.map(|(label, message)| {
            Button::new()
                .on_click(context.message(Message::Membership(message)))
                .content(label.to_owned())
        }))
}
