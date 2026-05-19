use crate::state::{get_or_create_room, AppState};
use crate::types::EncryptedPayload;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use futures_util::SinkExt;
use serde_json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Handle an incoming WebSocket upgrade request.
/// This function immediately upgrades the HTTP connection to WebSocket
/// and spawns the connection handler task.
pub async fn handle_socket_upgrade(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket_connection(socket, state))
}

/// Core WebSocket connection handler.
/// Orchestrates the full lifecycle of a single client connection:
/// - Splits the socket into sink (write) and stream (read)
/// - Reads the first message to extract room_code
/// - Spawns two async tasks: inbound and outbound message loops
/// - Manages graceful termination and waits for cleanup
async fn handle_socket_connection(socket: WebSocket, state: AppState) {
    if let Err(e) = manage_connection(socket, state).await {
        eprintln!("Connection error: {}", e);
    }
}

/// Manage the full lifecycle of a client connection.
/// Returns an error if the connection fails at any point.
async fn manage_connection(
    socket: WebSocket,
    state: AppState,
) -> Result<(), String> {
    let (sink, mut stream) = socket.split();

    // Read the first message to determine room_code
    let first_message = stream.next().await;
    let room_code = extract_room_code(first_message)?;

    // Retrieve or create the ChatRoom
    let room = get_or_create_room(&state, room_code);

    // Subscribe to the room's broadcast channel BEFORE spawning tasks.
    // This ensures we don't miss messages from other clients.
    let rx = room.tx.subscribe();

    // Flag to signal both tasks to terminate
    let should_terminate = Arc::new(AtomicBool::new(false));
    let terminate_clone_inbound = should_terminate.clone();
    let terminate_clone_outbound = should_terminate.clone();

    // Spawn the inbound message handler (reads subsequent messages and broadcasts them)
    let inbound_task = tokio::spawn(async move {
        inbound_loop(stream, room.tx.clone(), terminate_clone_inbound).await
    });

    // Spawn the outbound message handler (receives broadcasts and sends to client)
    let outbound_task = tokio::spawn(async move {
        outbound_loop(sink, rx, terminate_clone_outbound).await
    });

    // Wait for either task to complete, then signal termination and wait for the other.
    match tokio::try_join!(inbound_task, outbound_task) {
        Ok((inbound_res, outbound_res)) => {
            // Both tasks completed
            inbound_res?;
            outbound_res?;
            Ok(())
        }
        Err(e) => {
            // One or both tasks panicked
            should_terminate.store(true, Ordering::SeqCst);
            Err(format!("Task join error: {}", e))
        }
    }
}

/// Extract room_code from the first WebSocket message.
/// The client MUST send a valid JSON EncryptedPayload as the first message.
fn extract_room_code(
    first_message: Option<Result<axum::extract::ws::Message, axum::Error>>,
) -> Result<String, String> {
    match first_message {
        Some(Ok(axum::extract::ws::Message::Text(text))) => {
            serde_json::from_str::<EncryptedPayload>(&text)
                .map(|p| p.room_code)
                .map_err(|e| format!("Failed to parse payload: {}", e))
        }
        Some(Ok(axum::extract::ws::Message::Binary(_))) => {
            Err("Binary messages not supported; use Text format".to_string())
        }
        Some(Ok(axum::extract::ws::Message::Close(_))) => {
            Err("Connection closed before handshake".to_string())
        }
        Some(Ok(axum::extract::ws::Message::Ping(_))) => {
            Err("Ping received before handshake".to_string())
        }
        Some(Ok(axum::extract::ws::Message::Pong(_))) => {
            Err("Pong received before handshake".to_string())
        }
        Some(Err(e)) => Err(format!("WebSocket error: {}", e)),
        None => Err("Connection closed before first message".to_string()),
    }
}

/// Inbound message loop: read from the client and broadcast to the room.
/// This task runs until the connection is closed or an error occurs.
/// NOTE: The first message is used ONLY for room registration and is NOT broadcast.
async fn inbound_loop(
    mut stream: SplitStream<WebSocket>,
    tx: broadcast::Sender<EncryptedPayload>,
    should_terminate: Arc<AtomicBool>,
) -> Result<(), String> {
    loop {
        // Check if termination signal is set
        if should_terminate.load(Ordering::SeqCst) {
            break;
        }

        match stream.next().await {
            Some(Ok(axum::extract::ws::Message::Text(text))) => {
                // Parse the incoming message as an EncryptedPayload
                match serde_json::from_str::<EncryptedPayload>(&text) {
                    Ok(payload) => {
                        // Broadcast the payload to all subscribers in the room.
                        // If all receivers have dropped, ignore the error.
                        let _ = tx.send(payload);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse payload: {}", e);
                        // Continue to allow other clients to proceed
                    }
                }
            }
            Some(Ok(axum::extract::ws::Message::Close(_))) => {
                // Graceful close from client
                should_terminate.store(true, Ordering::SeqCst);
                break;
            }
            Some(Ok(_)) => {
                // Ignore other message types (Ping, Pong, Binary)
            }
            Some(Err(e)) => {
                eprintln!("WebSocket error: {}", e);
                should_terminate.store(true, Ordering::SeqCst);
                break;
            }
            None => {
                // Stream ended
                should_terminate.store(true, Ordering::SeqCst);
                break;
            }
        }
    }

    Ok(())
}

/// Outbound message loop: receive from the broadcast channel and send to the client.
/// This task runs until the connection is closed or an error occurs.
async fn outbound_loop(
    mut sink: SplitSink<WebSocket, axum::extract::ws::Message>,
    mut rx: broadcast::Receiver<EncryptedPayload>,
    should_terminate: Arc<AtomicBool>,
) -> Result<(), String> {
    loop {
        // Check if termination signal is set
        if should_terminate.load(Ordering::SeqCst) {
            break;
        }

        match rx.recv().await {
            Ok(payload) => {
                // Serialize the payload to JSON and send it to the client
                match serde_json::to_string(&payload) {
                    Ok(json_text) => {
                        if let Err(e) = sink
                            .send(axum::extract::ws::Message::Text(json_text))
                            .await
                        {
                            eprintln!("Failed to send message to client: {}", e);
                            should_terminate.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize payload: {}", e);
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // The receiver lagged and messages were dropped.
                // Continue to avoid disrupting the connection.
            }
            Err(broadcast::error::RecvError::Closed) => {
                // The broadcast channel was closed (room was dropped).
                should_terminate.store(true, Ordering::SeqCst);
                break;
            }
        }
    }

    // Attempt a graceful close before terminating
    let _ = sink.send(axum::extract::ws::Message::Close(None)).await;

    Ok(())
}
