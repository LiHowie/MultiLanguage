//! 只读取本用户主动准备的短期语音结果；不在私密输入或组句时触发。
use std::io::Read;

pub(super) fn take() -> Option<String> {
    let path = qingjian_platform::dirs::user_dir()?.join("last-speech.json");
    let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
    let age = std::time::SystemTime::now().duration_since(modified).ok()?;
    if age.as_secs() > 180 {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let mut data = String::new();
    std::fs::File::open(&path)
        .ok()?
        .take(16000)
        .read_to_string(&mut data)
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&data).ok()?;
    let text = value["text"].as_str()?.to_owned();
    if text.is_empty() || text.chars().count() > 4000 || text.contains('\0') {
        return None;
    }
    std::fs::remove_file(path).ok()?;
    Some(text)
}
