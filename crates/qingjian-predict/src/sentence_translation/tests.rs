//! 用本地 HTTP 服务验证实际请求、防抖、取消和风格，不调用收费 AI。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use qingjian_core::{Language, SentenceTranslationRequest, SentenceTranslator};

use super::{CloudSentenceTranslator, prompt};
use crate::{PredictConfig, TranslationStyle};

fn request(sequence: u64, text: &str) -> SentenceTranslationRequest {
    SentenceTranslationRequest {
        sequence,
        text: text.to_owned(),
        language: Language::English,
    }
}

#[test]
fn reply_rejects_empty_malformed_and_unbounded_output() {
    assert_eq!(
        prompt::parse(r#"{"translation":" I want to learn English. "}"#).as_deref(),
        Some("I want to learn English.")
    );
    for content in [r#"{"translation":"  "}"#, r#"{"sentence":"x"}"#, "not JSON"] {
        assert!(prompt::parse(content).is_none());
    }
    assert!(
        prompt::parse(&serde_json::json!({"translation": "x".repeat(4001)}).to_string()).is_none()
    );
}

#[test]
fn actual_request_uses_latest_sentence_language_and_style() {
    for style in TranslationStyle::ALL {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = PredictConfig {
            base_url: format!("http://{}/v1", listener.local_addr().unwrap()),
            api_key: Some("local-test".to_owned()),
            translation_debounce_ms: 100,
            translation_style: style,
            ..PredictConfig::default()
        };
        let server = std::thread::spawn(move || answer(listener, Duration::ZERO));
        let mut translator = CloudSentenceTranslator::new(&config).unwrap();
        translator.submit(request(1, "我想学习"));
        translator.submit(request(2, "我想学习英语"));
        let result = poll(&mut translator);
        assert_eq!(result.sequence, 2);
        assert_eq!(result.text.as_deref(), Some("I want to learn English."));
        let body = server.join().unwrap();
        let system = body["messages"][0]["content"].as_str().unwrap();
        assert!(system.contains(style.instruction()));
        let user: serde_json::Value =
            serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
        assert_eq!(
            user,
            serde_json::json!({"text":"我想学习英语", "target_language":"en"})
        );
        assert_eq!(body["model"], config.model);
        assert_eq!(body["max_tokens"], 2000);
    }
}

#[test]
fn cancel_before_debounce_sends_nothing() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let config = PredictConfig {
        base_url: format!("http://{}/v1", listener.local_addr().unwrap()),
        api_key: Some("local-test".to_owned()),
        translation_debounce_ms: 100,
        ..PredictConfig::default()
    };
    let mut translator = CloudSentenceTranslator::new(&config).unwrap();
    translator.submit(request(1, "我想学习英语"));
    translator.cancel();
    std::thread::sleep(Duration::from_millis(250));
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
    assert!(translator.poll().is_none());
}

#[test]
fn timeout_returns_failure_without_blocking_input_thread() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let config = PredictConfig {
        base_url: format!("http://{}/v1", listener.local_addr().unwrap()),
        api_key: Some("local-test".to_owned()),
        timeout_ms: 30,
        translation_debounce_ms: 100,
        ..PredictConfig::default()
    };
    let server = std::thread::spawn(move || answer(listener, Duration::from_millis(150)));
    let mut translator = CloudSentenceTranslator::new(&config).unwrap();
    translator.submit(request(9, "我想学习英语"));
    assert!(translator.poll().is_none());
    let result = poll(&mut translator);
    assert_eq!(result.sequence, 9);
    assert!(result.text.is_none());
    server.join().unwrap();
}

fn poll(translator: &mut CloudSentenceTranslator) -> qingjian_core::SentenceTranslationResult {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = translator.poll() {
            return result;
        }
        assert!(Instant::now() < deadline, "整句翻译没有返回");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn answer(listener: TcpListener, delay: Duration) -> serde_json::Value {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stream = loop {
        if let Ok((stream, _)) = listener.accept() {
            break stream;
        }
        assert!(Instant::now() < deadline, "没有收到请求");
        std::thread::sleep(Duration::from_millis(10));
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut reader = BufReader::new(&stream);
    let mut first = String::new();
    reader.read_line(&mut first).unwrap();
    assert!(first.starts_with("POST /v1/chat/completions "));
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse::<usize>().unwrap();
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    std::thread::sleep(delay);
    let response = serde_json::json!({
        "id":"local", "object":"chat.completion", "created":0, "model":"local",
        "choices":[{"index":0,"message":{"role":"assistant","content":"{\"translation\":\"I want to learn English.\"}"},"finish_reason":"stop"}]
    }).to_string();
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.len(),
        response
    );
    serde_json::from_slice(&body).unwrap()
}
