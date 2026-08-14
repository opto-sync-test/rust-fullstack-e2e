use std::time::Duration;

use opto_sync_client::{InMemoryStore, MutationStore, OptoSyncClient};
use opto_sync_rust_fullstack_e2e::background::{MultiplexBackgroundWorker, SyncLane};
use serde_json::{json, Value};

#[tokio::test]
async fn offline_rust_client_rebases_then_syncs_through_axum() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, opto_sync_rust_fullstack_e2e::router())
            .await
            .unwrap();
    });

    let http = reqwest::Client::new();
    let base_url = format!("http://{address}");
    let health: Value = http
        .get(format!("{base_url}/health"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(health["merge_engine"].as_str().unwrap().starts_with("0.2."));

    let mut client = OptoSyncClient::new(InMemoryStore::new()).without_clock();
    let mutation_id = client
        .queue_mutation(
            json!({
                "id": "doc-1",
                "title": "offline Rust edit",
                "updatedAt": "200",
                "metadata": {"clientOnly": true}
            })
            .to_string(),
        )
        .unwrap();
    let server_document: Value = http
        .get(format!("{base_url}/document"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let optimistic: Value =
        serde_json::from_str(&client.local_view(&server_document.to_string()).unwrap()).unwrap();
    assert_eq!(optimistic["title"], "offline Rust edit");

    let merged: Value = http
        .post(format!("{base_url}/merge"))
        .json(&json!({"incoming": optimistic}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(merged["metadata"]["serverOnly"], true);
    assert_eq!(merged["metadata"]["clientOnly"], true);
    assert!(client.store_mut().mark_synced(mutation_id));
    assert!(client.store().pending().is_empty());

    server.abort();
}

#[tokio::test]
async fn background_worker_reconnects_and_multiplexes_independent_lanes() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let worker = MultiplexBackgroundWorker::new(format!("http://{address}"))
        .with_retry_policy(20, Duration::from_millis(50));
    let drain = tokio::spawn(async move {
        worker
            .drain(&[
                SyncLane {
                    name: "desktop-window-one".to_string(),
                    incoming: json!({
                        "id": "doc-1",
                        "updatedAt": "200",
                        "metadata": {"laneOne": true}
                    }),
                },
                SyncLane {
                    name: "desktop-window-two".to_string(),
                    incoming: json!({
                        "id": "doc-1",
                        "updatedAt": "200",
                        "metadata": {"laneTwo": true}
                    }),
                },
            ])
            .await
    });

    tokio::time::sleep(Duration::from_millis(250)).await;
    let server = tokio::spawn(async move {
        axum::serve(listener, opto_sync_rust_fullstack_e2e::router())
            .await
            .unwrap();
    });

    let responses = drain.await.unwrap().unwrap();
    assert_eq!(
        responses
            .iter()
            .map(|(lane, _)| lane.as_str())
            .collect::<Vec<_>>(),
        ["desktop-window-one", "desktop-window-two"]
    );
    let authoritative: Value = reqwest::get(format!("http://{address}/document"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(authoritative["metadata"]["serverOnly"], true);
    assert_eq!(authoritative["metadata"]["laneOne"], true);
    assert_eq!(authoritative["metadata"]["laneTwo"], true);
    server.abort();
}
