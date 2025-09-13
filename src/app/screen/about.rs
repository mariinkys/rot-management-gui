// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{button, column, container, mouse_area, row, text};
use iced::{Alignment, Length, Subscription, Task};

use crate::app::style::{icon_button_style, icon_svg_style};
use crate::{fl, icons};

const AUTHOR_LINK: &str = "https://github.com/mariinkys";
const LICENSE_LINK: &str = "https://github.com/mariinkys/rot-management-gui/blob/main/LICENSE";
const REPOSITORY_LINK: &str = "https://github.com/mariinkys/rot-management-gui";
const ISSUES_LINK: &str = "https://github.com/mariinkys/rot-management-gui/issues";
const VERSION_LINK: &str = "https://github.com/mariinkys/rot-management-gui/releases";
const README_LINK: &str = "https://github.com/mariinkys/rot-management-gui/blob/main/README.md";

pub struct About {}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,
    /// Attempts to open a given URL
    LaunchURL(String),
}

pub enum Action {
    None,
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
                    mouse_area(text(env!("CARGO_PKG_AUTHORS")).style(text::primary))
                        .on_press(Message::LaunchURL(String::from(AUTHOR_LINK)))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("license")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    mouse_area(text(env!("CARGO_PKG_LICENSE")).style(text::primary))
                        .on_press(Message::LaunchURL(String::from(LICENSE_LINK)))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("repository")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    mouse_area(text(env!("CARGO_PKG_REPOSITORY")).style(text::primary))
                        .on_press(Message::LaunchURL(String::from(REPOSITORY_LINK)))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("issues")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    mouse_area(
                        text(format!("{}/issues", env!("CARGO_PKG_REPOSITORY")))
                            .style(text::primary)
                    )
                    .on_press(Message::LaunchURL(String::from(ISSUES_LINK)))
                ]
                .spacing(3.)
                .width(Length::Shrink),
                row![
                    text(fl!("version")).font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    mouse_area(text(env!("CARGO_PKG_VERSION")).style(text::primary))
                        .on_press(Message::LaunchURL(String::from(VERSION_LINK)))
                ]
                .spacing(3.)
                .width(Length::Shrink)
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill),
            column![
                text(fl!("attributions"))
                    .size(24)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                mouse_area(
                    text(fl!("pop-icons"))
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                        .style(text::primary)
                )
                .on_press(Message::LaunchURL(String::from(README_LINK))),
                mouse_area(
                    text(fl!("app-icon"))
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                        .style(text::primary)
                )
                .on_press(Message::LaunchURL(String::from(README_LINK))),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
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
            Message::LaunchURL(url) => {
                _ = open::that_detached(url);
                Action::None
            }
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
