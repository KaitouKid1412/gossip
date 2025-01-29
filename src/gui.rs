use eframe::egui;
use tokio::sync::mpsc::{Sender, UnboundedReceiver};
use anyhow::Result;
use crate::Command;

pub enum Selection {
    Create,
    Join,
}

pub fn show_selection_screen() -> Result<Selection> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(300.0, 200.0)),
        ..Default::default()
    };

    let selection = std::sync::Arc::new(std::sync::Mutex::new(None));
    let selection_clone = selection.clone();

    eframe::run_native(
        "Chat Selection",
        options,
        Box::new(move |_cc| {
            Box::new(SelectionApp {
                selection: selection_clone,
            })
        }),
    ).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    let result = selection.lock()
        .unwrap()
        .take()
        .expect("Selection was not made");
    Ok(result)
}

struct SelectionApp {
    selection: std::sync::Arc<std::sync::Mutex<Option<Selection>>>,
}

impl eframe::App for SelectionApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Choose an option");
                ui.add_space(20.0);
                
                if ui.button("Create private chatroom").clicked() {
                    *self.selection.lock().unwrap() = Some(Selection::Create);
                    frame.close();
                }
                
                ui.add_space(10.0);
                
                if ui.button("Join private chatroom").clicked() {
                    *self.selection.lock().unwrap() = Some(Selection::Join);
                    frame.close();
                }
            });
        });
    }
}

pub fn show_token_input_screen() -> Result<String> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 200.0)),
        ..Default::default()
    };

    let token = std::sync::Arc::new(std::sync::Mutex::new(None));
    let token_clone = token.clone();

    eframe::run_native(
        "Enter Join Token",
        options,
        Box::new(move |_cc| {
            Box::new(TokenInputApp {
                token: String::new(),
                result: token_clone,
            })
        }),
    ).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    let result = token.lock()
        .unwrap()
        .take()
        .expect("Token was not entered");
    Ok(result)
}

struct TokenInputApp {
    token: String,
    result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl eframe::App for TokenInputApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Enter Join Token");
                ui.add_space(20.0);
                
                ui.text_edit_singleline(&mut self.token);
                
                ui.add_space(10.0);
                
                if ui.button("Join").clicked() && !self.token.is_empty() {
                    *self.result.lock().unwrap() = Some(self.token.clone());
                    frame.close();
                }
            });
        });
    }
}

pub struct ChatWindow {
    messages: Vec<String>,
    input: String,
    sender: Sender<String>,
    receiver: UnboundedReceiver<String>,
    command_sender: Sender<Command>,
    join_token: String,
    is_chatting: bool,
    ticket: Option<String>,
    pending_message: Option<String>,
}

impl ChatWindow {
    pub fn new(
        sender: Sender<String>, 
        receiver: UnboundedReceiver<String>,
        command_sender: Sender<Command>,
    ) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            sender,
            receiver,
            command_sender,
            join_token: String::new(),
            is_chatting: false,
            ticket: None,
            pending_message: None,
        }
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(350.0, 450.0)),
            min_window_size: Some(egui::vec2(300.0, 300.0)),
            ..Default::default()
        };

        eframe::run_native(
            "Gossip",
            options,
            Box::new(|_cc| Box::new(self)),
        )
    }

    fn render_chat_ui(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();
        let input_height = 40.0;
        let content_height = available_height - input_height;

        ui.vertical(|ui| {
            // Top area with ticket and messages
            ui.allocate_ui(egui::vec2(ui.available_width(), content_height), |ui| {
                ui.vertical(|ui| {
                    // Ticket display
                    if let Some(ticket) = &self.ticket {
                        ui.horizontal(|ui| {
                            let available_width = ui.available_width();
                            let button_width = 70.0;
                            let spacing = 16.0;
                            let label_width = available_width - button_width - spacing;
                            
                            ui.group(|ui| {
                                ui.set_width(label_width);
                                ui.horizontal(|ui| {
                                    ui.label("Ticket: ");
                                    let truncated = if ticket.len() > 25 {
                                        format!("{}...", &ticket[..25])
                                    } else {
                                        ticket.clone()
                                    };
                                    ui.label(truncated);
                                });
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add_sized([button_width, 24.0], egui::Button::new("📋 Copy")).clicked() {
                                    ui.output_mut(|o| o.copied_text = ticket.clone());
                                }
                            });
                        });
                        ui.add_space(8.0);
                    }

                    // Messages area
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for message in &self.messages {
                                ui.label(message);
                            }
                        });
                });
            });

            // Input area at bottom with a small padding
            ui.add_space(5.0);  // Add a small padding before the input area
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    let text_edit = ui.add_sized(
                        [ui.available_width() - 60.0, 32.0],
                        egui::TextEdit::singleline(&mut self.input)
                            .hint_text("Type a message...")
                    );

                    let send_button = ui.add_sized(
                        [50.0, 32.0],
                        egui::Button::new("Send")
                    );

                    if (send_button.clicked() || 
                        (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) 
                        && !self.input.is_empty() 
                    {
                        self.pending_message = Some(self.input.clone());
                        self.input.clear();
                    }
                });
            });
            ui.add_space(20.0);  // Add a small padding after the input area
        });
    }

    fn render_join_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Gossip");
            ui.add_space(20.0);

            // Make Create button use full width
            let available_width = ui.available_width();
            if ui.add_sized([available_width, 32.0], egui::Button::new("Create Private Chatroom")).clicked() {
                if let Ok(_) = self.command_sender.try_send(Command::Open { topic: None }) {
                    self.is_chatting = true;
                }
            }

            ui.add_space(20.0);
            ui.label("Or join existing chat with token:");
            ui.add_space(10.0);

            // Join section remains the same
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                let button_width = 60.0;
                let spacing = 8.0;
                
                ui.add_sized(
                    [available_width - button_width - spacing, 32.0],
                    egui::TextEdit::singleline(&mut self.join_token)
                );
                
                if ui.add_sized([button_width, 32.0], egui::Button::new("Join")).clicked() && !self.join_token.is_empty() {
                    let cleaned_token = self.join_token.trim().to_string();
                    if let Ok(_) = self.command_sender.try_send(Command::Join { 
                        ticket: cleaned_token 
                    }) {
                        self.is_chatting = true;
                    }
                }
            });
        });
    }
}

impl eframe::App for ChatWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Check for new messages
        while let Ok(message) = self.receiver.try_recv() {
            if message.starts_with("Ticket:") {
                self.ticket = Some(message.trim_start_matches("Ticket:").to_string());
            } else {
                self.messages.push(message);
            }
        }

        // Try to send any pending message
        if let Some(msg) = self.pending_message.take() {
            if self.sender.try_send(msg.clone()).is_ok() {
                println!("Message sent successfully");
            } else {
                self.pending_message = Some(msg);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_chatting {
                self.render_chat_ui(ui);
            } else {
                self.render_join_ui(ui);
            }
        });

        ctx.request_repaint();
    }
}