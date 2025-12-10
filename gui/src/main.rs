use iced::widget::{column, text};
use iced::{Element, Sandbox, Settings};
use tracing_subscriber::EnvFilter;

#[derive(Default)]
struct App {
    message: String,
}

impl Sandbox for App {
    type Message = ();

    fn new() -> Self {
        Self {
            message: "LucAstra GUI placeholder. Enter commands, get responses.".into(),
        }
    }

    fn title(&self) -> String {
        "LucAstra".into()
    }

    fn update(&mut self, _message: Self::Message) {
        // No interactivity yet; wire commands to services in later steps.
    }

    fn view(&self) -> Element<Self::Message> {
        column![text(&self.message).size(24)].padding(16).into()
    }
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    App::run(Settings::default())
}
