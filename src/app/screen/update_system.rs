// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};

use crate::app::core::update_system::SystemUpdate;
use crate::app::core::utils::{reboot, reboot_pending};
use crate::app::style::{icon_button_style, primary_button_style};
use crate::app::widgets::toast::Toast;
use crate::{fl, icons};

pub struct UpdateSystem {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,

    /// Checks if the user has a pending reboot
    CheckReboot,
    /// Callback after checking if the user has a pending reboot
    RebootChecked(bool),

    /// Callback after attempting to fetch an update
    UpdateLoaded(Option<SystemUpdate>),
    /// Attempts to refresh the available updates
    RefreshAvailableUpdates,

    /// Calls the update function to apply an update
    Update,
    /// Callback after attempting to apply an update
    Updated(Result<(), anywho::Error>),

    /// Attempts to reboot the computer
    RebootNow,
    /// Callback presumably if reboot failed
    RebootCallback(Result<(), anywho::Error>),
}

pub enum State {
    Loading,
    Updating,
    PendingReboot,
    Ready { update: Option<SystemUpdate> },
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
    AddToastAndRun((Toast, Task<Message>)),
}

impl UpdateSystem {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(reboot_pending(), Message::RebootChecked),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let content = match &self.state {
            State::Ready { update, .. } => {
                if let Some(system_update) = update {
                    column![
                        Space::new(Length::Fill, Length::Fixed(35.)),
                        row![
                            text(fl!("system-update"))
                                .width(Length::Fill)
                                .size(18)
                                .font(iced::font::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                            button(text(fl!("update")))
                                .style(primary_button_style)
                                .on_press(Message::Update)
                        ],
                        container(
                            column![
                                row![
                                    text(fl!("version")).font(iced::font::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    }),
                                    text(&system_update.version)
                                ]
                                .spacing(2.),
                                row![
                                    text(fl!("commit")).font(iced::font::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    }),
                                    text(&system_update.commit)
                                ]
                                .spacing(2.),
                                row![
                                    text(fl!("gpg-signature")).font(iced::font::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    }),
                                    text(&system_update.gpg_signature)
                                ]
                                .spacing(2.),
                                row![
                                    text(fl!("sec-advisories")).font(iced::font::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    }),
                                    text(
                                        system_update
                                            .sec_advisories
                                            .clone()
                                            .unwrap_or(String::from("None"))
                                    )
                                ]
                                .spacing(2.),
                                row![
                                    text(fl!("diff")).font(iced::font::Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Default::default()
                                    }),
                                    text(&system_update.diff)
                                ]
                                .spacing(2.)
                            ]
                            .spacing(3.)
                        )
                        .style(container::rounded_box)
                        .padding(20.)
                    ]
                    .padding(20.)
                    .spacing(5.)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                } else {
                    column![
                        Space::new(Length::Fill, Length::Fixed(35.)),
                        text(fl!("no-updates"))
                            .width(Length::Fill)
                            .size(18)
                            .font(iced::font::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    ]
                    .padding(20.)
                    .spacing(5.)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                }
            }
            State::PendingReboot => column![
                Space::new(Length::Fill, Length::Fixed(35.)),
                text(fl!("reboot-required"))
                    .size(24)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .align_x(Alignment::Center),
                text(fl!("reboot-message"))
                    .size(18)
                    .align_x(Alignment::Center),
                button(text(fl!("reboot-now")))
                    .style(primary_button_style)
                    .on_press(Message::RebootNow)
            ]
            .padding(20.)
            .spacing(10.)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Alignment::Center),
            State::Updating => {
                return container(
                    column![text(fl!("updating")), text(fl!("updating-warning"))]
                        .spacing(3.)
                        .align_x(Alignment::Center),
                )
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
            }
            _ => {
                return container(text(fl!("checking-updates")))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        };

        let main_content = container(content)
            .align_x(Alignment::Center)
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

        let refresh_button = container(
            button(icons::get_icon("view-refresh-symbolic", 18))
                .on_press(Message::CheckReboot)
                .style(icon_button_style),
        )
        .align_x(Alignment::End)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        iced::widget::stack![main_content, back_button, refresh_button].into()
    }
    #[allow(clippy::only_used_in_recursion)]
    pub fn update(&mut self, message: Message, now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
            Message::CheckReboot => {
                self.state = State::Loading;
                Action::Run(Task::perform(reboot_pending(), Message::RebootChecked))
            }
            Message::RebootChecked(result) => match result {
                true => {
                    self.state = State::PendingReboot;
                    Action::None
                }
                false => self.update(Message::RefreshAvailableUpdates, now),
            },
            Message::UpdateLoaded(system_update) => {
                self.state = State::Ready {
                    update: system_update,
                };
                Action::None
            }
            Message::RefreshAvailableUpdates => {
                self.state = State::Loading;
                Action::Run(Task::perform(SystemUpdate::check(), Message::UpdateLoaded))
            }
            Message::Update => {
                self.state = State::Updating;
                Action::Run(Task::perform(SystemUpdate::update(), Message::Updated))
            }
            Message::Updated(result) => match result {
                Ok(_) => self.update(Message::CheckReboot, now),
                Err(err) => Action::AddToastAndRun((
                    Toast::error_toast(err),
                    Task::perform(reboot_pending(), Message::RebootChecked),
                )),
            },
            Message::RebootNow => Action::Run(Task::perform(reboot(), Message::RebootCallback)),
            Message::RebootCallback(result) => match result {
                Ok(_) => self.update(Message::CheckReboot, now),
                Err(err) => Action::AddToast(Toast::error_toast(err)),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
