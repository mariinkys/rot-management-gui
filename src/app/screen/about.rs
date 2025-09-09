// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};

use crate::app::style::icon_button_style;
use crate::{fl, icons};

pub struct About {}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,
}

pub enum Action {
    Back,
}

impl About {
    pub fn new() -> (Self, Task<Message>) {
        (Self {}, Task::none())
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let content = column![
            text(fl!("about"))
                .size(32)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .align_x(Alignment::Center)
                .width(Length::Fill),
            column![
                row![
                    text(fl!("author")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(env!("CARGO_PKG_AUTHORS"))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("license")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(env!("CARGO_PKG_LICENSE"))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("repository")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(env!("CARGO_PKG_REPOSITORY"))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("issues")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(format!("{}/issues", env!("CARGO_PKG_REPOSITORY")))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("version")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    text(env!("CARGO_PKG_VERSION"))
                ]
                .spacing(3.)
                .width(Length::Shrink)
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill),
        ]
        .width(Length::Fill)
        .padding(20.)
        .spacing(30.);

        let main_content = container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill);

        let back_button = container(
            button(icons::get_icon("go-previous-symbolic", 18))
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

    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
