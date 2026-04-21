use eframe::egui;
use reqwest;
use rfd;
use std::fs;
use std::sync::Arc;
use tokio;

const DEFAULT_INTERVAL: u64 = 5;
const HTTP_TIMEOUT_SECS: u64 = 10;
const MAX_IDLE_CONNECTIONS: usize = 10;

#[derive(serde::Serialize)]
struct WebhookPayload {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum SendMode {
    Interval,
    Frequency,
}

struct App {
    user_pairs: Vec<(String, String)>, // (username, avatar_url) - webhook URLは共通使用
    pair_input: String,
    use_shuffle: bool,
    urls: String,
    message: String,
    send_mode: SendMode,
    value: String,
    status: String,
    error: String,
    is_running: bool,
    stop_sender: Option<tokio::sync::mpsc::Sender<()>>,
    client: reqwest::Client,
}

impl Default for App {
    fn default() -> Self {
        Self {
            user_pairs: vec![("Webhook Bot".to_string(), String::new())],
            pair_input: String::new(),
            use_shuffle: false,
            urls: String::new(),
            message: "@everyone".to_string(),
            send_mode: SendMode::Interval,
            value: "5".to_string(),
            status: "Ready".to_string(),
            error: String::new(),
            is_running: false,
            stop_sender: None,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
                .pool_max_idle_per_host(MAX_IDLE_CONNECTIONS)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🚀 SEKITEI CANNNON 2026 WORKING");
            });

            ui.separator();

            // Status
            ui.horizontal(|ui| {
                ui.label("📊 Status:");
                ui.colored_label(
                    if self.is_running { egui::Color32::GREEN } else { egui::Color32::GRAY },
                    &self.status
                );
            });

            ui.separator();

            // Bot Settings
            egui::CollapsingHeader::new("🤖 Bot Settings")
                .default_open(false)
                .show(ui, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Shuffle user pairs:");
                            ui.checkbox(&mut self.use_shuffle, "");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("User Pairs (Name|Icon URL):");
                            if ui.button("📁 Import from File").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    match fs::read_to_string(&path) {
                                        Ok(content) => {
                                            self.user_pairs.clear();
                                            let mut count = 0;
                                            for line in content.lines() {
                                                let line = line.trim();
                                                if !line.is_empty() && !line.starts_with('#') {
                                                    let parts: Vec<&str> = line.splitn(2, '|').collect();
                                                    if parts.len() == 2 {
                                                        self.user_pairs.push((parts[0].to_string(), parts[1].to_string()));
                                                        count += 1;
                                                    }
                                                }
                                            }
                                            self.status = format!("Pairs imported ({})", count);
                                            self.error.clear();
                                        }
                                        Err(e) => {
                                            self.error = format!("Failed to import pairs: {}", e);
                                        }
                                    }
                                }
                            }
                            if ui.button("🧪 Test Avatar").clicked() {
                                // テスト用のwebhook送信
                                if !self.urls.is_empty() && !self.user_pairs.is_empty() {
                                    let urls: Vec<String> = self.urls.lines()
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    let client = self.client.clone();
                                    let message = "🧪 Avatar Test Message".to_string();
                                    let bot_name = self.user_pairs[0].0.clone();
                                    let avatar_url = self.user_pairs[0].1.clone();

                                    tokio::spawn(async move {
                                        for url in &urls {
                                            if let Err(e) = send_webhook(&client, url, &message, &bot_name, &avatar_url).await {
                                                eprintln!("Test failed: {}", e);
                                            }
                                        }
                                    });
                                    self.status = "Avatar test sent - check console output".to_string();
                                } else {
                                    self.error = "Configure webhook URLs and user pairs first".to_string();
                                }
                            }
                        });
                        
                        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                            let mut remove_index = None;
                            for (i, (name, avatar)) in self.user_pairs.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}:", i + 1));
                                    ui.text_edit_singleline(name);
                                    ui.text_edit_singleline(avatar);
                                    
                                    // アイコン状態を表示
                                    if avatar.is_empty() {
                                        ui.colored_label(egui::Color32::YELLOW, "⚠️ No icon");
                                    } else if avatar.starts_with("http://") || avatar.starts_with("https://") {
                                        ui.colored_label(egui::Color32::GREEN, "✅ OK");
                                    } else {
                                        ui.colored_label(egui::Color32::RED, "❌ Invalid");
                                    }
                                    
                                    if ui.button("X").clicked() {
                                        remove_index = Some(i);
                                    }
                                });
                            }
                            if let Some(idx) = remove_index {
                                self.user_pairs.remove(idx);
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.pair_input);
                            if ui.button("Add Pair").clicked() {
                                let parts: Vec<&str> = self.pair_input.splitn(2, '|').collect();
                                if parts.len() == 2 {
                                    self.user_pairs.push((parts[0].to_string(), parts[1].to_string()));
                                    self.pair_input.clear();
                                }
                            }
                        });
                    });
                });

            ui.separator();

            // Webhook Settings
            egui::CollapsingHeader::new("🔗 Webhook Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Webhook URLs (one per line):");
                            if ui.button("📁 Import from File").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    match fs::read_to_string(&path) {
                                        Ok(content) => {
                                            let urls: Vec<String> = content
                                                .lines()
                                                .map(|line| line.trim())
                                                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                                                .map(|line| line.to_string())
                                                .collect();
                                            self.urls = urls.join("\n");
                                            self.status = format!("URLs imported ({})", urls.len());
                                            self.error.clear();
                                        }
                                        Err(e) => {
                                            self.error = format!("Failed to import URLs: {}", e);
                                        }
                                    }
                                }
                            }
                        });
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.urls)
                                    .hint_text("https://discord.com/api/webhooks/...\nhttps://discord.com/api/webhooks/...")
                                    .desired_rows(5)
                            );
                        });
                    });
                });

            ui.separator();

            // Message Settings
            egui::CollapsingHeader::new("💬 Message Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Message:");
                        });
                        ui.text_edit_multiline(&mut self.message);
                    });
                });

            ui.separator();

            // Send Settings
            egui::CollapsingHeader::new("⏱️ Send Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Send Mode:");
                            ui.radio_value(&mut self.send_mode, SendMode::Interval, "Interval (seconds)");
                            ui.radio_value(&mut self.send_mode, SendMode::Frequency, "Frequency (times per second)");
                            ui.text_edit_singleline(&mut self.value);
                        });
                    });
                });

            ui.separator();

            // Control Buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if self.is_running { "⏹️ Stop" } else { "▶️ Start" };
                    if ui.button(button_text).clicked() {
                        if self.is_running {
                            self.stop();
                            self.status = "Stopped".to_string();
                        } else {
                            self.start();
                            self.status = "Running".to_string();
                        }
                    }
                });
            });

            if self.is_running {
                ui.label("📡 Sending webhooks...");
            }
            
            if !self.error.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error);
            }
        });
    }
}

impl App {
    fn start(&mut self) {
        if self.is_running {
            return;
        }

        let urls: Vec<String> = self.urls.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if urls.is_empty() {
            self.error = "No webhook URLs configured".to_string();
            return;
        }
        
        if self.user_pairs.is_empty() {
            self.error = "No user pairs configured".to_string();
            return;
        }

        let message = self.message.clone();
        let user_pairs = Arc::new(self.user_pairs.clone());
        let use_shuffle = self.use_shuffle;
        let interval = match self.send_mode {
            SendMode::Interval => {
                match self.value.parse::<u64>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        self.error = "Invalid interval value. Using default 5 seconds".to_string();
                        DEFAULT_INTERVAL
                    }
                }
            },
            SendMode::Frequency => {
                match self.value.parse::<f64>() {
                    Ok(freq) if freq > 0.0 => (1.0 / freq) as u64,
                    _ => {
                        self.error = "Invalid frequency value. Using default 0.2 Hz".to_string();
                        DEFAULT_INTERVAL
                    }
                }
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        self.stop_sender = Some(tx);

        self.is_running = true;
        self.error.clear();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut pair_index = 0;
            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {
                        let (bot_name, avatar_url) = if use_shuffle {
                            let idx = fastrand::usize(0..user_pairs.len());
                            (&user_pairs[idx].0, &user_pairs[idx].1)
                        } else {
                            pair_index = pair_index % user_pairs.len();
                            let pair = &user_pairs[pair_index];
                            pair_index += 1;
                            (&pair.0, &pair.1)
                        };

                        for url in &urls {
                            if let Err(e) = send_webhook(&client, url, &message, bot_name, avatar_url).await {
                                eprintln!("Error sending to {}: {}", url, e);
                            }
                        }
                    }
                }
            }
        });
    }

    fn stop(&mut self) {
        if let Some(tx) = self.stop_sender.take() {
            let _ = tx.try_send(());
        }
        self.is_running = false;
    }
}

async fn send_webhook(
    client: &reqwest::Client,
    url: &str,
    message: &str,
    bot_name: &str,
    avatar_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // より強力なキャッシュ回避：ランダムなクエリパラメータを追加
    let avatar_url_with_cache_bust = if !avatar_url.is_empty() {
        let random_id = fastrand::u64(..);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        format!("{}?cache_bust={}_{}", avatar_url, timestamp, random_id)
    } else {
        avatar_url.to_string()
    };

    let payload = WebhookPayload {
        content: message.to_string(),
        username: if bot_name.is_empty() { None } else { Some(bot_name.to_string()) },
        avatar_url: if avatar_url_with_cache_bust.is_empty() { None } else { Some(avatar_url_with_cache_bust.clone()) },
    };

    // 詳細なデバッグ出力
    println!("🚀 Sending Discord Webhook:");
    println!("   Webhook URL: {}", url);
    println!("   Username: {}", bot_name);
    println!("   Avatar URL: {}", avatar_url_with_cache_bust);
    println!("   Message: {}", message);
    println!("   Payload JSON: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

    let response = client.post(url).json(&payload).send().await?;
    let status = response.status();

    if status.is_success() {
        println!("✅ Webhook sent successfully (Status: {})", status);
        let response_text = response.text().await.unwrap_or_default();
        if !response_text.is_empty() {
            println!("   Response: {}", response_text);
        }
    } else {
        let error_text = response.text().await.unwrap_or_default();
        println!("❌ Failed to send webhook:");
        println!("   Status: {}", status);
        println!("   Error: {}", error_text);

        // Discord APIエラーの詳細を表示
        if status == reqwest::StatusCode::BAD_REQUEST {
            println!("   💡 Bad Request - avatar_urlの形式が正しくない可能性があります");
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            println!("   💡 Unauthorized - Webhook URLが無効です");
        } else if status == reqwest::StatusCode::FORBIDDEN {
            println!("   💡 Forbidden - Webhookの権限が不足しています");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SEKITEI CANNNON 2026 WORKING",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(App::default())
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Try to use system fonts that support CJK characters
    if let Ok(font_data) = std::fs::read("assets/fonts/NotoSansCJK-Regular.ttf") {
        fonts.font_data.insert("noto_sans_cjk".to_owned(), egui::FontData {
            font: std::borrow::Cow::from(font_data),
            index: 0,
            tweak: Default::default(),
        });
        
        fonts.families.entry(egui::FontFamily::Proportional).or_insert_with(Vec::new).insert(0, "noto_sans_cjk".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_insert_with(Vec::new).insert(0, "noto_sans_cjk".to_owned());
    }
    
    ctx.set_fonts(fonts);
}