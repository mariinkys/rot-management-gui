// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{Space, button, column, container, responsive, text};
use iced::{Alignment, Length, Subscription, Task};

use crate::app::core::rollback::rollback;
use crate::app::core::utils::{reboot, reboot_pending};
use crate::app::style::{icon_button_style, icon_svg_style, primary_button_style};
use crate::app::widgets::spinners::circular::Circular;
use crate::app::widgets::spinners::easing;
use crate::app::widgets::spinners::linear::Linear;
use crate::app::widgets::toast::Toast;
use crate::{fl, icons};

pub struct Rollback {
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

    /// Calls the rollback function to apply a rollback
    Rollback,
    /// Callback after attempting to apply a rollback
    RollbackCompleted(Result<(), anywho::Error>),

    /// Attempts to reboot the computer
    RebootNow,
    /// Callback presumably if reboot failed
    RebootCallback(Result<(), anywho::Error>),
}

pub enum State {
    Loading,
    ApplyingRollback,
    PendingReboot,
    Ready,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
    AddToastAndRun((Toast, Task<Message>)),
}

impl Rollback {
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
            State::Ready => column![
                Space::new(Length::Fill, Length::Fixed(35.)),
                text(fl!("rollback"))
                    .size(24)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .align_x(Alignment::Center),
                text(fl!("rollback-confirmation"))
                    .size(18)
                    .align_x(Alignment::Center),
                button(text(fl!("rollback")))
                    .style(primary_button_style)
                    .on_press(Message::Rollback)
            ]
            .padding(20.)
            .spacing(10.)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Alignment::Center),
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
            State::ApplyingRollback => {
                return container(
                    column![
                        text(fl!("applying-rollback")),
                        text(fl!("rollback-warning")),
                        responsive(|bounds| {
                            container(
                                Linear::new()
                                    .easing(&easing::STANDARD_DECELERATE)
                                    .cycle_duration(std::time::Duration::from_secs_f32(10.0))
                                    .width(Length::Fixed(bounds.width / 2.0)),
                            )
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                            .into()
                        })
                    ]
                    .spacing(10.)
                    .height(Length::Shrink)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                )
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
            }
            _ => {
                return container(
                    column![
                        text(fl!("loading")),
                        Circular::new()
                            .easing(&easing::EMPHASIZED)
                            .cycle_duration(std::time::Duration::from_secs_f32(5.0))
                    ]
                    .spacing(10.)
                    .align_x(Alignment::Center),
                )
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
            button(icons::get_icon("go-previous-symbolic", 18).style(icon_svg_style))
                .on_press(Message::Back)
                .style(icon_button_style),
        )
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        let refresh_button = container(
            button(icons::get_icon("view-refresh-symbolic", 18).style(icon_svg_style))
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
                false => {
                    self.state = State::Ready;
                    Action::None
                }
            },
            Message::Rollback => {
                self.state = State::ApplyingRollback;
                Action::Run(Task::perform(rollback(), Message::RollbackCompleted))
            }
            Message::RollbackCompleted(result) => match result {
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
