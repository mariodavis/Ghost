#!/usr/bin/env python3
"""
Ghost Client - Simple CLI for communicating through Ghost engine
Handles JSON serialization and nonce generation automatically
"""

import asyncio
import json
import websockets
import sys
import uuid
from datetime import datetime

class GhostClient:
    def __init__(self, server_url, room_code):
        self.server_url = server_url
        self.room_code = room_code
        self.ws = None
        self.message_count = 0
        
    async def connect(self):
        """Connect to Ghost server and register to room"""
        try:
            self.ws = await websockets.connect(self.server_url)
            
            # Send registration message (first message)
            registration = {
                "room_code": self.room_code,
                "ciphertext": "registration",
                "nonce": str(uuid.uuid4())
            }
            await self.ws.send(json.dumps(registration))
            print(f"\n✅ Connected to room '{self.room_code}'")
            print("📝 Type your message and press Enter (type 'quit' to exit)\n")
            return True
        except Exception as e:
            print(f"❌ Connection error: {e}")
            return False
    
    async def send_message(self, text):
        """Send a message to the room"""
        if not self.ws:
            print("❌ Not connected")
            return False
        
        try:
            self.message_count += 1
            message = {
                "room_code": self.room_code,
                "ciphertext": text,
                "nonce": f"msg-{self.message_count}-{uuid.uuid4().hex[:8]}"
            }
            await self.ws.send(json.dumps(message))
            return True
        except Exception as e:
            print(f"❌ Send error: {e}")
            return False
    
    async def receive_messages(self):
        """Listen for incoming messages"""
        if not self.ws:
            return
        
        try:
            async for message in self.ws:
                data = json.loads(message)
                timestamp = datetime.now().strftime("%H:%M:%S")
                print(f"\n[{timestamp}] 📨 {data['ciphertext']}")
                print("You: ", end="", flush=True)
        except asyncio.CancelledError:
            pass
        except Exception as e:
            print(f"\n❌ Receive error: {e}")
    
    async def run(self):
        """Main event loop"""
        if not await self.connect():
            return
        
        # Start receiving messages in background
        receive_task = asyncio.create_task(self.receive_messages())
        
        try:
            loop = asyncio.get_event_loop()
            while True:
                # Prompt for input (non-blocking)
                text = await loop.run_in_executor(None, input, "You: ")
                
                if text.lower() == 'quit':
                    print("\n👋 Goodbye!")
                    break
                
                if text.strip():
                    await self.send_message(text)
        except KeyboardInterrupt:
            print("\n\n👋 Goodbye!")
        finally:
            receive_task.cancel()
            if self.ws:
                await self.ws.close()


async def main():
    if len(sys.argv) < 2:
        print("Usage: python3 ghost-client.py <room-code> [server-url]")
        print("\nExample:")
        print("  python3 ghost-client.py my-room")
        print("  python3 ghost-client.py secret-chat ws://192.168.1.100:8080/ws")
        sys.exit(1)
    
    room_code = sys.argv[1]
    server_url = sys.argv[2] if len(sys.argv) > 2 else "ws://localhost:8080/ws"
    
    client = GhostClient(server_url, room_code)
    await client.run()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Exiting...")
        sys.exit(0)
