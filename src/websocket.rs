use crate::{error::RpcError, models::SubscribeUTXORequest, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio_stream::StreamExt;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    addresses: String, // Comma-separated addresses
}

/// WebSocket endpoint for UTXO subscription
pub async fn subscribe_utxo(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    let addresses: Vec<String> = query
        .addresses
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if addresses.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "No addresses provided",
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_utxo_subscription(socket, addresses, state))
}

async fn handle_utxo_subscription(
    mut socket: WebSocket,
    addresses: Vec<String>,
    state: AppState,
) {
    tracing::info!("New UTXO subscription for {} addresses", addresses.len());

    // Subscribe to Kaspa UTXO changes
    let stream_result = state
        .kaspa_client
        .subscribe_utxo_changes(addresses.clone())
        .await;

    let mut stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({
                        "error": format!("Failed to subscribe: {}", e)
                    })
                    .to_string(),
                ))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    // Send initial connection success
    let _ = socket
        .send(Message::Text(
            serde_json::json!({
                "status": "subscribed",
                "addresses": addresses,
            })
            .to_string(),
        ))
        .await;

    // Forward UTXO change notifications to WebSocket client
    while let Some(response) = stream.next().await {
        match response {
            Ok(kaspad_response) => {
                // Check if it's a UTXO changed notification
                if let Some(crate::client::proto::kaspad_response::Payload::UtxosChangedNotification(
                    notification,
                )) = kaspad_response.payload
                {
                    let message = serde_json::json!({
                        "type": "utxo_changed",
                        "added": notification.added.iter().map(|entry| {
                            serde_json::json!({
                                "address": entry.address,
                                "outpoint": entry.outpoint.as_ref().map(|op| {
                                    serde_json::json!({
                                        "transaction_id": op.transaction_id,
                                        "index": op.index,
                                    })
                                }),
                                "utxo_entry": entry.utxo_entry.as_ref().map(|utxo| {
                                    serde_json::json!({
                                        "amount": utxo.amount,
                                        "script_public_key": utxo.script_public_key.as_ref().map(|s| s.script_public_key.clone()),
                                        "block_daa_score": utxo.block_daa_score,
                                        "is_coinbase": utxo.is_coinbase,
                                    })
                                }),
                            })
                        }).collect::<Vec<_>>(),
                        "removed": notification.removed.iter().map(|entry| {
                            serde_json::json!({
                                "address": entry.address,
                                "outpoint": entry.outpoint.as_ref().map(|op| {
                                    serde_json::json!({
                                        "transaction_id": op.transaction_id,
                                        "index": op.index,
                                    })
                                }),
                            })
                        }).collect::<Vec<_>>(),
                    });

                    if socket
                        .send(Message::Text(message.to_string()))
                        .await
                        .is_err()
                    {
                        tracing::warn!("Client disconnected");
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({
                            "error": format!("Stream error: {}", e)
                        })
                        .to_string(),
                    ))
                    .await;
                break;
            }
        }
    }

    let _ = socket.close().await;
    tracing::info!("UTXO subscription closed");
}
