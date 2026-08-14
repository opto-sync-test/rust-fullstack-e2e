use std::time::Duration;

use reqwest::Client;
use serde_json::{json, Value};
use tokio::{task::JoinSet, time::sleep};

#[derive(Clone, Debug)]
pub struct SyncLane {
    pub name: String,
    pub incoming: Value,
}

/// Desktop/server background worker: one wake drains independent lanes in
/// parallel, while any partial transport failure replays the immutable batch.
/// Reconciliation is idempotent, so a response lost after the server applies a
/// lane cannot duplicate domain state on the next attempt.
#[derive(Clone, Debug)]
pub struct MultiplexBackgroundWorker {
    endpoint: String,
    max_attempts: usize,
    retry_delay: Duration,
    http: Client,
}

impl MultiplexBackgroundWorker {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            max_attempts: 8,
            retry_delay: Duration::from_millis(100),
            http: Client::new(),
        }
    }

    pub fn with_retry_policy(mut self, max_attempts: usize, retry_delay: Duration) -> Self {
        assert!(max_attempts > 0, "max_attempts must be positive");
        self.max_attempts = max_attempts;
        self.retry_delay = retry_delay;
        self
    }

    pub async fn drain(&self, lanes: &[SyncLane]) -> Result<Vec<(String, Value)>, String> {
        if lanes.is_empty() {
            return Ok(Vec::new());
        }
        let mut last_error = String::new();
        for attempt in 1..=self.max_attempts {
            let mut tasks = JoinSet::new();
            for lane in lanes.iter().cloned() {
                let http = self.http.clone();
                let endpoint = self.endpoint.clone();
                tasks.spawn(async move {
                    let response = http
                        .post(format!("{endpoint}/merge"))
                        .json(&json!({"incoming": lane.incoming}))
                        .send()
                        .await
                        .map_err(|error| error.to_string())?
                        .error_for_status()
                        .map_err(|error| error.to_string())?
                        .json::<Value>()
                        .await
                        .map_err(|error| error.to_string())?;
                    Ok::<_, String>((lane.name, response))
                });
            }

            let mut responses = Vec::with_capacity(lanes.len());
            let mut failed = false;
            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok(Ok(response)) => responses.push(response),
                    Ok(Err(error)) => {
                        last_error = error;
                        failed = true;
                    }
                    Err(error) => {
                        last_error = error.to_string();
                        failed = true;
                    }
                }
            }
            if !failed && responses.len() == lanes.len() {
                responses.sort_by(|left, right| left.0.cmp(&right.0));
                return Ok(responses);
            }
            if attempt < self.max_attempts {
                sleep(self.retry_delay).await;
            }
        }
        Err(format!(
            "background sync exhausted {} attempts: {last_error}",
            self.max_attempts
        ))
    }
}
