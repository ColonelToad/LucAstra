use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Color, Element, Length, Sandbox, Settings};
use lucastra_app::SystemState;
use lucastra_core::{Command, CommandPayload, ResponsePayload};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    OpenFileManager,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    system_state: SystemState,
    chat_input: String,
    chat_history: Vec<ChatMessage>,
    command_counter: usize,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        let system_state = match SystemState::new() {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Failed to initialize system: {}", e);
                SystemState::default()
            }
        };

        Self {
            system_state,
            chat_input: String::new(),
            chat_history: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome to LucAstra OS! Ask me anything.".to_string(),
            }],
            command_counter: 0,
        }
    }

    fn title(&self) -> String {
        "LucAstra OS - Desktop".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::InputChanged(value) => {
                self.chat_input = value;
            }
            Message::SendMessage => {
                if self.chat_input.trim().is_empty() {
                    return;
                }

                let user_message = self.chat_input.clone();
                self.chat_history.push(ChatMessage {
                    role: "user".to_string(),
                    content: user_message.clone(),
                });
                self.chat_input.clear();

                // Process command
                self.command_counter += 1;
                let cmd = Command {
                    id: format!("gui-cmd-{}", self.command_counter),
                    payload: CommandPayload::Query {
                        text: user_message,
                        use_rag: Some(true),
                    },
                };

                let response = match self.system_state.handle_command(cmd) {
                    Ok(resp) => match resp.payload {
                        ResponsePayload::Success(text) => text,
                        ResponsePayload::Status(status) => status,
                        ResponsePayload::Devices(devices) => devices.join("\n"),
                        ResponsePayload::Files(files) => {
                            files.iter().map(|f| f.path.clone()).collect::<Vec<_>>().join("\n")
                        }
                        ResponsePayload::Content(bytes) => {
                            String::from_utf8_lossy(&bytes).to_string()
                        }
                        ResponsePayload::SearchResults(results) => {
                            results.iter().map(|r| format!("{}: {}", r.path, r.snippet)).collect::<Vec<_>>().join("\n")
                        }
                        ResponsePayload::Error(err) => format!("Error: {}", err),
                    },
                    Err(e) => format!("System error: {}", e),
                };

                self.chat_history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response,
                });
            }
            Message::OpenFileManager => {
                self.chat_history.push(ChatMessage {
                    role: "system".to_string(),
                    content: "File manager opened (placeholder).".to_string(),
                });
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // Taskbar at bottom
        let taskbar = container(
            row![
                button(text("File Manager")).on_press(Message::OpenFileManager),
                text("  |  LucAstra OS").size(14),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .padding(10)
        .width(Length::Fill)
        .style(taskbar_style);

        // Chat history
        let mut chat_messages = Column::new().spacing(10).padding(10);
        for msg in &self.chat_history {
            let role_label = match msg.role.as_str() {
                "user" => "You:",
                "assistant" => "LucAstra:",
                "system" => "System:",
                _ => "Unknown:",
            };
            let message_color = match msg.role.as_str() {
                "user" => Color::from_rgb(0.3, 0.5, 0.9),
                "assistant" => Color::from_rgb(0.2, 0.8, 0.4),
                "system" => Color::from_rgb(0.6, 0.6, 0.6),
                _ => Color::WHITE,
            };

            chat_messages = chat_messages.push(
                column![
                    text(role_label)
                        .size(12)
                        .style(message_color),
                    text(&msg.content).size(16),
                ]
                .spacing(2),
            );
        }

        let chat_scroll = scrollable(chat_messages).height(Length::Fill);

        // Input area
        let input_row = row![
            text_input("Type your message...", &self.chat_input)
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage)
                .padding(10)
                .size(16),
            button(text("Send").size(16))
                .on_press(Message::SendMessage)
                .padding(10),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center);

        // Main layout
        column![chat_scroll, input_row, taskbar,]
            .spacing(0)
            .into()
    }
}

fn taskbar_style(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    App::run(Settings::default())
}
