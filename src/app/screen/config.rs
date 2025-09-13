// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{button, column, container, pick_list, text};
use iced::{Alignment, Length, Subscription, Task, Theme};

use crate::app::core::config::ApplicationTheme;
use crate::app::style::{icon_button_style, icon_svg_style};
use crate::{fl, icons};

pub struct Config {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,

    /// Pick List Change Theme Callback
    ChangedTheme(ApplicationTheme),
}

pub struct State {
    config: crate::app::core::config::Config,
}

pub enum Action {
    Back,
    ChangedTheme(ApplicationTheme),
}

impl Config {
    pub fn new(current_config: crate::app::core::config::Config) -> (Self, Task<Message>) {
        (
            Self {
                state: State {
                    config: current_config,
                },
            },
            Task::none(),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let content = column![
            text(fl!("configuration"))
                .size(32)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .align_x(Alignment::Center)
                .width(Length::Fill),
            column![
                text(fl!("theme")),
                pick_list(
                    Theme::ALL,
                    Some::<Theme>(self.state.config.theme.clone().into()),
                    |t| {
                        Message::ChangedTheme(ApplicationTheme::try_from(&t).unwrap_or_default())
                    }
                )
                .width(Length::Fill)
            ]
            .spacing(3.)
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

    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
            Message::ChangedTheme(application_theme) => {
                self.state.config.theme = application_theme.clone();
                Action::ChangedTheme(application_theme)
            }
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
