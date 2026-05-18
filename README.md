GhostBytes is an ultra-fast, zero-knowledge, anonymous real-time chat broker powered by a high-performance **Rust** backend. 

Operating as a **"Blind Broker"**, the system does not utilize a database, stores no persistent logs, and has zero visibility into message contents. It routes end-to-end encrypted data payloads between anonymous clients entirely in-memory via WebSockets.

---

## 🧠 System Architecture Overview

```text
[Client A (Sender)] ───► Encrypted JSON (Ciphertext) ───► [GhostBytes Server]
                                                                  │
                                                          (In-Memory Routing)
                                                                  │
[Client B (Receiver)] ◄─── Encrypted JSON (Ciphertext) ◄──────────┘

    Anonymous Access: Authentication relies strictly on a dynamic room code. No personal identifiers, credentials, or registration metrics are collected or stored.

    Client-Side Cryptography: Plaintext data is encrypted directly within the user's browser using the Web Crypto API (AES-GCM-256) prior to network transmission.

    Memory-Isolated Routing: The Rust engine acts as an encrypted packet relay. Incoming data is ingested via an asynchronous WebSocket stream, mapped via an in-memory concurrent map (dashmap), and broadcasted to connected peers using lightweight tokio channels.

    Ephemerality: Messages exist exclusively in volatile RAM during transit and are instantly purged upon delivery. Rooms are dynamically deallocated from memory when all connected clients disconnect.

🛠️ Installation
Prerequisites

The system requires the Rust toolchain. Install it via your package manager or directly from rustup.rs:
Bash

curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

Setup

    Clone the repository:
    Bash

git clone [https://github.com/yourusername/ghostbytes.git](https://github.com/yourusername/ghostbytes.git)
cd ghostbytes

Compile the optimized production binary:
Bash

    cargo build --release

   *The compiled executable will be generated at `./target/release/ghostbytes`.*

---

## 🚀 Usage & Testing

### 1. Execute the Server
Launch the underlying engine to listen on all interfaces via port `8080`:
```bash
./target/release/ghostbytes

2. Live Connection Simulation

To verify real-time, zero-knowledge routing behavior over WebSockets, you can utilize a command-line socket tester like wscat.

    Step A: Install the utility:
    Bash

    npm install -g wscat


* **Step B:** Instantiate the first client connection (User 1) and subscribe to room `101`:
  ```bash
  wscat -c ws://localhost:8080/ws
  

Send the structural routing payload:
JSON

{"room_code":"101","ciphertext":"Z2hvc3RidXRlcw==","nonce":"MTIzNDU2Nzg5MDEy"}

    Step C: Open a separate terminal window to connect the second client (User 2) to the same room:
    Bash

    wscat -c ws://localhost:8080/ws

  Send the identical initialization payload to establish the channel. 

Any subsequent structured payloads sent through either socket terminal will be instantly mirrored to peer screens with microsecond latency and zero disk footprints.

---

## 🛡️ Production Deployment Security Notes
* **TLS Termination:** Always deploy this service behind a reverse proxy (e.g., Caddy, Nginx) configured for `WSS` (WebSocket Secure) to safeguard transaction metadata over the public internet.
* **Log Mitigation:** Configure upstream ingress controllers or proxies to explicitly discard application connection logs to guarantee complete endpoint metadata anonymity.
