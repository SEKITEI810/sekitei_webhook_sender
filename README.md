# SEKITEI CANNNON 2026 WORKING

A standalone GUI application for sending messages to multiple Discord webhooks simultaneously.

## Features

- Send messages to multiple Discord webhooks at once
- Configurable sending frequency: interval (seconds) or frequency (times per second)
- Customizable app name
- **Multiple bot user pairs with shuffle option**: Define multiple (bot name, avatar) pairs and choose to shuffle on each message or cycle sequentially
- File import for webhook URL lists and user pairs
- Modern GUI with collapsible sections, status indicators, and intuitive layout
- Standalone executable (no installation required)

## Installation

Download the `sekitei-cannon-2026-working.exe` file from the releases and run it directly.

### 日本語フォント対応

日本語表示が文字化けしている場合は、以下の対応を行ってください：

1. Noto Sans CJK をダウンロード
   - [Google Fonts](https://fonts.google.com/?query=noto%20sans%20cjk) から「Noto Sans CJK JP」をダウンロード
   
2. フォントを `assets/fonts/` ディレクトリに配置
   - ファイル名を `NotoSansCJK-Regular.ttf` に変更
   
3. アプリを再起動

または、Windows に標準で含まれるメイリオやYu Gothicが自動的に使用されます。

## Usage

1. Run the executable: `sekitei-cannon-2026-working.exe`

2. In the GUI:
   - View status indicator showing current state
   - Set bot appearance (name and avatar, expandable section)
   - **Manage bot user pairs** (Bot Settings section):
     - Add multiple (bot name, avatar URL) pairs
     - Import pairs from file (format: `BotName|AvatarURL`)
     - Toggle shuffle mode for random pair selection on each message
     - Or use sequential mode to cycle through pairs in order
   - Import webhook URLs from file or enter manually (scrollable text area)
   - Compose your message
   - Choose sending mode and timing
   - Start/stop sending with prominent control button

## Examples

### Interval Mode
- App Name: "My Custom Cannon"
- URLs: (multiple webhook URLs)
- Message: "Status update"
- Mode: Interval (seconds)
- Value: 10

This sends "Status update" to all URLs every 10 seconds.

### Frequency Mode
- URLs: (multiple webhook URLs)
- Message: "Heartbeat"
- Mode: Frequency (times per second)
- Value: 0.5

This sends "Heartbeat" to all URLs twice every second (every 2 seconds).

### User Pair Shuffling Mode
- Bot User Pairs:
  - Pair 1: "BotName1", "https://avatar.url1"
  - Pair 2: "BotName2", "https://avatar.url2"
  - Pair 3: "BotName3", "https://avatar.url3"
- Shuffle Mode: Enabled
- Message: "Rotating identity message"
- Mode: Interval (seconds)
- Value: 5

This sends "Rotating identity message" to all URLs every 5 seconds, **randomly selecting a different bot name and avatar for each message**. Without shuffle enabled, it would cycle through pairs sequentially instead.

## Building from Source

If you want to build from source:

```bash
cargo build --release
```

The executable will be in `target/release/sekitei-cannon-2026-working.exe`

## Dependencies

- reqwest: For HTTP requests
- tokio: For async runtime
- serde: For JSON serialization
- eframe/egui: For GUI framework
- winres: For Windows resources (icon and metadata)