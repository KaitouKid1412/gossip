# P2P Chat Application

A peer-to-peer chat application built with Rust using iroh-gossip for decentralized communication.

## Key Differences from WhatsApp/Telegram
Unlike WhatsApp or Telegram which rely on centralized servers to route messages, this application:
- Uses peer-to-peer communication
- Doesn't store messages on any central server
- Doesn't require user accounts or phone numbers
- Messages are shared directly between participants using gossip protocol
- Works without internet connectivity (on local networks)

## Installation & Setup

### 1. Install Rust
1. Visit [rustup.rs](https://rustup.rs)
2. Follow the installation instructions for your operating system:
   - **Windows**: Download and run rustup-init.exe
   - **macOS/Linux**: Run the following in your terminal:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
3. Restart your terminal after installation

### 2. Run the Chat Application
1. Clone this repository
2. Navigate to the project directory
3. Build and run the application:
   ```bash
   cargo run
   ```
4. The GUI will launch automatically
5. To start chatting:
   - One user should click "Create Room" to generate a ticket
   - Copy the ticket at the top of the window and share it with whoever should join the chat.
   - Other users can join by pasting the ticket and clicking "Join Room"
