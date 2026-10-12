//! 整句翻译的云端实现，与云联想共用连接设置，独立开启。

use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use qingjian_core::{SentenceTranslationRequest, SentenceTranslationResult, SentenceTranslator};

use super::worker::SentenceWorker;
use crate::chat_client::ChatClient;
use crate::{PredictConfig, PredictError};

pub struct CloudSentenceTranslator {
    /// 有界唤醒通道，连续输入不堆积请求。
    wake: SyncSender<()>,

    pending: Arc<Mutex<Option<SentenceTranslationRequest>>>,

    responses: Receiver<SentenceTranslationResult>,
}

impl CloudSentenceTranslator {
    pub fn new(config: &PredictConfig) -> Result<Self, PredictError> {
        let key = config
            .resolve_api_key()
            .ok_or_else(|| PredictError::MissingApiKey(config.api_key_env.clone()))?;
        let (wake, receiver) = mpsc::sync_channel(1);
        let (responses, response_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(None));
        let worker = SentenceWorker {
            wake: receiver,
            pending: pending.clone(),
            responses,
            client: ChatClient::new(config, key),
            // 整句比联想多等一会儿，减少逐键请求；用户可在配置中调整。
            debounce: Duration::from_millis(config.translation_debounce_ms.clamp(100, 5000)),
            style: config.translation_style,
            membership: config
                .membership_service
                .then(|| crate::MembershipClient::new(config))
                .transpose()
                .map_err(|_| PredictError::EmptyReply)?,
        };
        std::thread::Builder::new()
            .name("qingjian-sentence-translation".to_owned())
            .spawn(move || {
                if let Err(error) = worker.run() {
                    tracing::warn!(%error, "整句翻译线程退出");
                }
            })?;
        Ok(Self {
            wake,
            pending,
            responses: response_rx,
        })
    }
}

impl SentenceTranslator for CloudSentenceTranslator {
    fn submit(&mut self, request: SentenceTranslationRequest) {
        *self.pending.lock().unwrap() = Some(request);
        let _ = self.wake.try_send(());
    }

    fn cancel(&mut self) {
        *self.pending.lock().unwrap() = None;
        let _ = self.wake.try_send(());
    }

    fn poll(&mut self) -> Option<SentenceTranslationResult> {
        self.responses.try_recv().ok()
    }
}
