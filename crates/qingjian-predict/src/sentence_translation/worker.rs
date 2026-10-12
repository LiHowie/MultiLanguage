//! 整句翻译线程：单个最新请求槽位、防抖与异步结果，不记录原文。

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use qingjian_core::{SentenceTranslationRequest, SentenceTranslationResult};

use super::prompt;
use crate::chat_client::ChatClient;
use crate::{PredictError, TranslationStyle};

pub(super) struct SentenceWorker {
    pub wake: Receiver<()>,

    pub pending: Arc<Mutex<Option<SentenceTranslationRequest>>>,

    pub responses: Sender<SentenceTranslationResult>,

    pub client: ChatClient,

    pub debounce: Duration,

    pub style: TranslationStyle,

    pub membership: Option<crate::MembershipClient>,
}

impl SentenceWorker {
    pub fn run(self) -> Result<(), PredictError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        while self.wake.recv().is_ok() {
            loop {
                match self.wake.recv_timeout(self.debounce) {
                    Ok(()) => continue,
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => return Ok(()),
                }
            }
            let Some(request) = self.pending.lock().unwrap().take() else {
                continue;
            };
            let text = if let Some(service) = &self.membership {
                runtime
                    .block_on(service.translate(
                        &request.text,
                        request.language.code(),
                        self.style.code(),
                    ))
                    .ok()
            } else {
                let result = runtime.block_on(self.client.chat_translation(
                    &prompt::system(self.style),
                    &prompt::user(&request),
                    2000,
                ));
                match result {
                    Ok(content) => prompt::parse(&content),
                    Err(error) => {
                        tracing::warn!(sequence = request.sequence, %error, "整句翻译失败");
                        None
                    }
                }
            };
            if self
                .responses
                .send(SentenceTranslationResult {
                    sequence: request.sequence,
                    text,
                })
                .is_err()
            {
                return Ok(());
            }
        }
        Ok(())
    }
}
