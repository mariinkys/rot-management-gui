// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{Rule, Space, button, column, container, row, scrollable, text, tooltip};
use iced::{Alignment, Element, Length, Subscription, Task};

use crate::app::core::system_status::Deployment;
use crate::app::core::utils::{reboot, reboot_pending};
use crate::app::style::{icon_button_style, primary_button_style};
use crate::app::widgets::toast::Toast;
use crate::{fl, icons};

pub struct SystemStatus {
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

    /// Loads the deployments (rpm-ostree status)
    LoadDeployments,
    /// Callback after loading all the deployments
    DeploymentsLoaded(Result<Vec<Deployment>, anywho::Error>),

    /// Request to manage layered packages
    //ManageLayeredPackages,

    /// Ask to pin a deployment
    PinDeployment(Deployment),
    /// Ask to unpin a deployment
    UnpinDeployment(Deployment),
    /// Callback after pinning unpinning a deployment
    DeploymentPinChanged(Result<(), anywho::Error>),

    /// Attempts to reboot the computer
    RebootNow,
    /// Callback presumably if reboot failed
    RebootCallback(Result<(), anywho::Error>),
}

pub enum State {
    Loading,
    PendingReboot,
    Ready { deployments: Vec<Deployment> },
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
    AddToastAndRun((Toast, Task<Message>)),
}

impl SystemStatus {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(reboot_pending(), Message::RebootChecked),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let content: Element<Message> = match &self.state {
            State::Loading => container(text(fl!("loading")))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
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
            .align_x(Alignment::Center)
            .into(),
            State::Ready { deployments } => {
                if deployments.is_empty() {
                    column![
                        Space::new(Length::Fill, Length::Fixed(35.)),
                        text(fl!("no-deployments-error"))
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
                    .into()
                } else {
                    let mut deployment_cards = column![]
                        .align_x(Alignment::Center)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .spacing(10.);

                    for deployment in deployments {
                        deployment_cards = deployment_cards.push(deployment_card(deployment));
                    }

                    column![
                        Space::new(Length::Fill, Length::Fixed(35.)),
                        row![
                            text(fl!("system-status"))
                                .width(Length::Fill)
                                .size(18)
                                .font(iced::font::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                            // button(text(fl!("manage-layered-packages")))
                            //     .style(primary_button_style)
                            //     .on_press(Message::ManageLayeredPackages)
                        ],
                        scrollable(deployment_cards),
                    ]
                    .padding(20.)
                    .spacing(5.)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .into()
                }
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
                false => self.update(Message::LoadDeployments, now),
            },
            Message::LoadDeployments => Action::Run(Task::perform(
                Deployment::get_all(),
                Message::DeploymentsLoaded,
            )),
            Message::DeploymentsLoaded(deployments) => match deployments {
                Ok(deployments) => {
                    self.state = State::Ready { deployments };
                    Action::None
                }
                Err(err) => {
                    self.state = State::Ready {
                        deployments: Vec::new(),
                    };
                    Action::AddToast(Toast::error_toast(err))
                }
            },
            // Message::ManageLayeredPackages => match self.pending_reboot {
            //     true => Action::AddToast(Toast::error_toast(
            //         "Can't manage layered packages with a pending reboot",
            //     )),
            //     false => Action::AddToast(Toast::warning_toast("Feature in development")),
            // },
            Message::PinDeployment(deployment) => {
                if !deployment.is_pinned {
                    return Action::Run(Task::perform(
                        Deployment::pin_deployment(deployment.index),
                        Message::DeploymentPinChanged,
                    ));
                }
                Action::None
            }
            Message::UnpinDeployment(deployment) => {
                if deployment.is_pinned {
                    return Action::Run(Task::perform(
                        Deployment::unpin_deployment(deployment.index),
                        Message::DeploymentPinChanged,
                    ));
                }
                Action::None
            }
            Message::DeploymentPinChanged(result) => match result {
                Ok(_) => self.update(Message::LoadDeployments, now),
                Err(err) => Action::AddToastAndRun((
                    Toast::error_toast(err),
                    Task::perform(Deployment::get_all(), Message::DeploymentsLoaded),
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

fn deployment_card<'a>(deployment: &'a Deployment) -> Element<'a, Message> {
    let pin_button = match deployment.is_pinned {
        true => tooltip(
            button(icons::get_icon("unpin-symbolic", 18))
                .style(icon_button_style)
                .width(Length::Shrink)
                .on_press(Message::UnpinDeployment(deployment.clone())),
            container(text(fl!("unpin-deployment")))
                .padding(5.)
                .style(container::secondary),
            tooltip::Position::Top,
        ),
        false => tooltip(
            button(icons::get_icon("pin-symbolic", 18))
                .style(icon_button_style)
                .width(Length::Shrink)
                .on_press(Message::PinDeployment(deployment.clone())),
            container(text(fl!("pin-deployment")))
                .padding(5)
                .style(container::secondary),
            tooltip::Position::Top,
        ),
    };

    let card_header = row![
        text(format!(
            "({}) {} - {}",
            &deployment.index, &deployment.name, &deployment.version
        ))
        .width(Length::Fill)
        .size(18)
        .font(iced::font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        pin_button
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .padding(10.);

    let deployment_content = column![
        row![
            text(fl!("version")).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text(&deployment.version)
        ]
        .spacing(2.),
        row![
            text(fl!("commit")).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text(&deployment.base_commit)
        ]
        .spacing(2.),
        row![
            text(fl!("gpg-signature")).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text(&deployment.gpg_signature)
        ]
        .spacing(2.),
        row![
            text(fl!("layered-packages")).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text(&deployment.layered_packages)
        ]
        .spacing(2.)
    ]
    .spacing(3.)
    .padding(10.);

    container(column![
        card_header,
        Rule::horizontal(1.),
        deployment_content
    ])
    .style(container::rounded_box)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
