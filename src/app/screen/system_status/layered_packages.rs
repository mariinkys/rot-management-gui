use iced::time::Instant;
use iced::widget::{button, container};
use iced::{Alignment, Element, Length};
use iced::{Task, widget::text};

use crate::app::style::{icon_button_style, icon_svg_style};
use crate::app::{core::system_status::Deployment, widgets::toast::Toast};
use crate::icons;

pub struct LayeredPackages {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,
}

pub enum State {
    Ready { current_deployment: Deployment },
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl LayeredPackages {
    pub fn new(current_deployment: Deployment) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Ready { current_deployment },
            },
            Task::none(),
        )
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
        }
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let content: Element<Message> = text("Layered Packages Content").into();

        let main_content = container(content)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let back_button = container(
            button(icons::get_icon("go-previous-symbolic", 18).style(icon_svg_style))
                .on_press(Message::Back)
                .style(icon_button_style),
        )
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        iced::widget::stack![main_content, back_button].into()
    }
}
