use serde::{Deserialize, Serialize};

/// EncryptedPayload represents an end-to-end encrypted network message.
/// All fields are opaque to the broker (Ghost server).
/// The broker never decrypts, verifies, or interprets the ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedPayload {
    /// Routing identifier used purely for in-memory room lookup.
    /// The broker never interprets the content.
    pub room_code: String,

    /// Opaque ciphertext encrypted by the client's browser.
    /// The broker never decrypts or inspects this field.
    pub ciphertext: String,

    /// Cryptographic Initialization Vector (IV).
    /// Required by the client for decryption; opaque to the broker.
    pub nonce: String,
}

impl EncryptedPayload {
    /// Construct a new EncryptedPayload from raw network components.
    pub fn new(room_code: String, ciphertext: String, nonce: String) -> Self {
        Self {
            room_code,
            ciphertext,
            nonce,
        }
    }
}
