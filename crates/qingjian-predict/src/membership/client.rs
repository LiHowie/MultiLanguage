//! 只向已配置的魔多服务发送用户主动提交的内容，拒绝不安全的远程 HTTP。
use crate::{PredictConfig, chat_client};
use serde_json::{Value, json};
use std::time::Duration;

pub struct MembershipClient {
    client: reqwest::Client,

    base: String,

    token: String,

    consent: bool,
}

impl MembershipClient {
    pub fn new(config: &PredictConfig) -> Result<Self, String> {
        let url = reqwest::Url::parse(&config.base_url).map_err(|_| "服务地址无效")?;
        let local = matches!(
            url.host_str(),
            Some("127.0.0.1" | "localhost" | "::1" | "10.0.2.2")
        );
        if url.scheme() != "https" && !(url.scheme() == "http" && local) {
            return Err("远程服务必须使用 HTTPS".to_owned());
        }
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("服务地址不能包含账号、参数或片段".to_owned());
        }
        Ok(Self {
            client: chat_client::http_client(&config.base_url),
            base: config.base_url.trim_end_matches('/').to_owned(),
            token: config.resolve_api_key().unwrap_or_default(),
            consent: config.cloud_consent,
        })
    }

    pub async fn request(&self, route: &str, data: Option<Value>) -> Result<Value, String> {
        let url = format!("{}{route}", self.base);
        let mut request = if let Some(data) = data {
            self.client.post(url).json(&data)
        } else {
            self.client.get(url)
        };
        if !self.token.is_empty() {
            request = request.bearer_auth(&self.token);
        }
        let response = request
            .timeout(Duration::from_secs(50))
            .send()
            .await
            .map_err(|_| "网络请求失败，请检查服务地址和连接")?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(|_| "服务返回不完整")?;
        if bytes.len() > 100_000 {
            return Err("服务响应过大".to_owned());
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| "服务响应格式无效")?;
        if !status.is_success() {
            return Err(value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("服务请求失败")
                .chars()
                .take(250)
                .collect());
        }
        Ok(value)
    }

    pub async fn translate(
        &self,
        text: &str,
        language: &str,
        style: &str,
    ) -> Result<String, String> {
        if !self.consent {
            return Err("请先同意云端学习".to_owned());
        }
        let value = self
            .request(
                "/translate",
                Some(json!({"input":text,"source":"zh","target":language,
            "style":style,"cloudConsent":true})),
            )
            .await?;
        value["translation"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() < 16000)
            .map(str::to_owned)
            .ok_or_else(|| "翻译结果为空".to_owned())
    }

    pub fn block(&self, route: &str, data: Option<Value>) -> Result<Value, String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| "网络线程初始化失败")?
            .block_on(self.request(route, data))
    }
}
