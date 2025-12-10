use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Column};
use iced::{Alignment, Color, Element, Length, Sandbox, Settings, Size};
use lucastra_app::SystemState;
use lucastra_config::{self, Config};
use lucastra_core::{Command, CommandPayload, ResponsePayload};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    OpenFileManager,
    OpenSettings,
    CloseSettings,
    SaveSettings,
    ClearError,
    DismissToast(usize),
    UpdateSetting(SettingChange),
}

#[derive(Debug, Clone)]
pub enum SettingChange {
    ServerUrl(String),
    ModelSize(String),
    Temperature(String),
    MaxTokens(String),
    Theme(String),
    AutoStart(bool),
    UseGpu(bool),
    WindowWidth(String),
    WindowHeight(String),
    FontSize(String),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct NoticeToast {
    pub id: usize,
    pub message: String,
}

pub struct App {
    system_state: SystemState,
    chat_input: String,
    chat_history: Vec<ChatMessage>,
    command_counter: usize,
    settings_open: bool,
    temp_config: Config,
    error: Option<String>,
    notices: Vec<NoticeToast>,
    next_notice_id: usize,
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

        let temp_config = system_state.get_config().clone();

        Self {
            system_state,
            chat_input: String::new(),
            chat_history: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome to LucAstra OS! Ask me anything.".to_string(),
            }],
            command_counter: 0,
            settings_open: false,
            temp_config,
            error: None,
            notices: Vec::new(),
            next_notice_id: 0,
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
                        ResponsePayload::Files(files) => files
                            .iter()
                            .map(|f| f.path.clone())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        ResponsePayload::Content(bytes) => {
                            String::from_utf8_lossy(&bytes).to_string()
                        }
                        ResponsePayload::SearchResults(results) => results
                            .iter()
                            .map(|r| format!("{}: {}", r.path, r.snippet))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        ResponsePayload::Error(err) => format!("Error: {}", err),
                    },
                    Err(e) => {
                        self.error = Some(format!("Command failed: {}", e));
                        format!("System error: {}", e)
                    }
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
                self.push_notice("File manager opened (placeholder)");
            }
            Message::OpenSettings => {
                self.settings_open = true;
                self.temp_config = self.system_state.get_config().clone();
            }
            Message::CloseSettings => {
                self.settings_open = false;
            }
            Message::SaveSettings => {
                match self.system_state.update_config(self.temp_config.clone()) {
                    Ok(_) => self.chat_history.push(ChatMessage {
                        role: "system".to_string(),
                        content: "Settings saved.".to_string(),
                    }),
                    Err(e) => {
                        self.error = Some(format!("Failed to save settings: {}", e));
                        self.chat_history.push(ChatMessage {
                            role: "system".to_string(),
                            content: format!("Failed to save settings: {}", e),
                        });
                    }
                }
                self.settings_open = false;
                if self.error.is_none() {
                    self.push_notice("Settings saved.");
                }
            }
            Message::ClearError => {
                self.error = None;
            }
            Message::DismissToast(id) => {
                self.notices.retain(|toast| toast.id != id);
            }
            Message::UpdateSetting(change) => match change {
                SettingChange::ServerUrl(url) => {
                    self.temp_config.llm.server_url = url;
                }
                SettingChange::ModelSize(model) => {
                    self.temp_config.llm.model_size = model;
                }
                SettingChange::Temperature(val) => {
                    if let Ok(t) = val.parse::<f32>() {
                        self.temp_config.llm.temperature = t;
                    }
                }
                SettingChange::MaxTokens(val) => {
                    if let Ok(tokens) = val.parse::<u32>() {
                        self.temp_config.llm.max_tokens = tokens;
                    }
                }
                SettingChange::Theme(theme) => {
                    self.temp_config.gui.theme = theme;
                }
                SettingChange::AutoStart(enabled) => {
                    self.temp_config.llm.auto_start = enabled;
                }
                SettingChange::UseGpu(enabled) => {
                    self.temp_config.llm.use_gpu = enabled;
                }
                SettingChange::WindowWidth(val) => {
                    if let Ok(width) = val.parse::<u32>() {
                        self.temp_config.gui.window_width = width;
                    }
                }
                SettingChange::WindowHeight(val) => {
                    if let Ok(height) = val.parse::<u32>() {
                        self.temp_config.gui.window_height = height;
                    }
                }
                SettingChange::FontSize(val) => {
                    if let Ok(size) = val.parse::<u16>() {
                        self.temp_config.gui.font_size = size;
                    }
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.settings_open {
            return self.view_settings();
        }

        let taskbar = container(
            row![
                button(text("File Manager")).on_press(Message::OpenFileManager),
                button(text("Settings")).on_press(Message::OpenSettings),
                text("  |  LucAstra OS").size(14),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .padding(10)
        .width(Length::Fill)
        .style(taskbar_style);

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
                    text(role_label).size(12).style(message_color),
                    text(&msg.content).size(16),
                ]
                .spacing(2),
            );
        }

        let chat_scroll = scrollable(chat_messages).height(Length::Fill);
        let toasts = self.build_toasts();

        let content = if let Some(toasts) = toasts {
            row![
                chat_scroll.width(Length::Fill),
                column![toasts]
                    .width(Length::Shrink)
                    .padding([10, 10, 10, 0])
                    .align_items(Alignment::End),
            ]
            .spacing(16)
            .height(Length::Fill)
        } else {
            row![chat_scroll.width(Length::Fill)].height(Length::Fill)
        };

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

        let error_banner: Option<Element<Message>> = self.error.as_ref().map(|msg| {
            container(
                row![
                    text("Error").style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.8, 0.8))),
                    text(msg).style(iced::theme::Text::Color(Color::WHITE)),
                    button(text("Dismiss")).on_press(Message::ClearError),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .padding(10)
            .width(Length::Fill)
            .style(error_banner_style)
            .into()
        });

        let base = column![content, input_row, taskbar].spacing(0).into();

        if let Some(banner) = error_banner {
            column![banner, base].into()
        } else {
            base
        }
    }
}

impl App {
    fn view_settings(&self) -> Element<'_, Message> {
        let model_sizes = vec!["7b".to_string(), "13b".to_string(), "70b".to_string()];

        let error_banner: Option<Element<Message>> = self.error.as_ref().map(|msg| {
            container(
                row![
                    text("Error").style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.8, 0.8))),
                    text(msg).style(iced::theme::Text::Color(Color::WHITE)),
                    button(text("Dismiss")).on_press(Message::ClearError),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .padding(10)
            .width(Length::Fill)
            .style(error_banner_style)
            .into()
        });

        let settings_content = column![
            text("LucAstra Settings").size(24),
            text("LLM Configuration").size(18),
            row![
                text("Server URL:").width(Length::Fixed(140.0)),
                text_input("http://localhost:8000", &self.temp_config.llm.server_url)
                    .on_input(|v| Message::UpdateSetting(SettingChange::ServerUrl(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Model Size:").width(Length::Fixed(140.0)),
                pick_list(model_sizes.clone(), Some(self.temp_config.llm.model_size.clone()), |v| {
                    Message::UpdateSetting(SettingChange::ModelSize(v))
                }),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Temperature:").width(Length::Fixed(140.0)),
                text_input("0.7", &format!("{:.2}", self.temp_config.llm.temperature))
                    .on_input(|v| Message::UpdateSetting(SettingChange::Temperature(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Max Tokens:").width(Length::Fixed(140.0)),
                text_input("2048", &self.temp_config.llm.max_tokens.to_string())
                    .on_input(|v| Message::UpdateSetting(SettingChange::MaxTokens(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Auto-start:").width(Length::Fixed(140.0)),
                checkbox("", self.temp_config.llm.auto_start)
                    .on_toggle(|v| Message::UpdateSetting(SettingChange::AutoStart(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("GPU Acceleration:").width(Length::Fixed(140.0)),
                checkbox("", self.temp_config.llm.use_gpu)
                    .on_toggle(|v| Message::UpdateSetting(SettingChange::UseGpu(v))),
            ]
            .spacing(10)
            .padding(5),
            text("GUI Configuration").size(18),
            row![
                text("Theme:").width(Length::Fixed(140.0)),
                text_input("dark", &self.temp_config.gui.theme)
                    .on_input(|v| Message::UpdateSetting(SettingChange::Theme(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Window Width:").width(Length::Fixed(140.0)),
                text_input("1280", &self.temp_config.gui.window_width.to_string())
                    .on_input(|v| Message::UpdateSetting(SettingChange::WindowWidth(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Window Height:").width(Length::Fixed(140.0)),
                text_input("800", &self.temp_config.gui.window_height.to_string())
                    .on_input(|v| Message::UpdateSetting(SettingChange::WindowHeight(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Font Size:").width(Length::Fixed(140.0)),
                text_input("16", &self.temp_config.gui.font_size.to_string())
                    .on_input(|v| Message::UpdateSetting(SettingChange::FontSize(v))),
            ]
            .spacing(10)
            .padding(5),
            row![
                button(text("Save")).on_press(Message::SaveSettings),
                button(text("Cancel")).on_press(Message::CloseSettings),
            ]
            .spacing(10)
            .padding(10),
        ]
        .spacing(12)
        .padding(20);

        let toasts = self.build_toasts();

        let settings_body = if let Some(toasts) = toasts {
            row![
                scrollable(settings_content).width(Length::Fill),
                column![toasts]
                    .width(Length::Shrink)
                    .padding([10, 10, 10, 0])
                    .align_items(Alignment::End),
            ]
            .spacing(16)
        } else {
            row![scrollable(settings_content).width(Length::Fill)]
        };

        let settings_view = container(settings_body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10);

        if let Some(banner) = error_banner {
            column![banner, settings_view].into()
        } else {
            settings_view.into()
        }
    }

    fn push_notice(&mut self, message: impl Into<String>) {
        let id = self.next_notice_id;
        self.next_notice_id += 1;
        self.notices.push(NoticeToast {
            id,
            message: message.into(),
        });

        if self.notices.len() > 3 {
            self.notices.remove(0);
        }
    }

    fn build_toasts(&self) -> Option<Column<'_, Message>> {
        if self.notices.is_empty() {
            return None;
        }

        let mut stack = Column::new().spacing(8).align_items(Alignment::End);

        for notice in &self.notices {
            stack = stack.push(
                container(
                    row![
                        text("Info").style(iced::theme::Text::Color(Color::from_rgb(0.8, 0.9, 1.0))),
                        text(&notice.message).style(iced::theme::Text::Color(Color::WHITE)),
                        button(text("Dismiss")).on_press(Message::DismissToast(notice.id)),
                    ]
                    .spacing(8)
                    .align_items(Alignment::Center),
                )
                .padding(8)
                .width(Length::Shrink)
                .style(toast_style),
            );
        }

        Some(stack)
    }
}

fn taskbar_style(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn error_banner_style(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.5, 0.1, 0.1))),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn toast_style(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.2, 0.35))),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

fn main() -> iced::Result {
    let logs_dir = lucastra_config::get_logs_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./logs"));

    std::fs::create_dir_all(&logs_dir).ok();

    let file_appender = tracing_appender::rolling::daily(logs_dir, "lucastra-gui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(file_layer)
        .with(stdout_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set logger");

    let config = Config::load().unwrap_or_default();

    let settings = Settings {
        window: iced::window::Settings {
            size: Size::new(config.gui.window_width as f32, config.gui.window_height as f32),
            ..Default::default()
        },
        ..Default::default()
    };

    App::run(settings)
}
