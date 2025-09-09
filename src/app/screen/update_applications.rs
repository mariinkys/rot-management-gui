// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Length, Subscription, Task};

use crate::app::core::update_applications::{
    Application, ApplicationStatus, UpdateError, UpdateResult,
};
use crate::app::style::{AccordionButtonPosition, icon_button_style, primary_button_style};
use crate::app::utils::ui::{AccordionButtonStatus, AccordionIcon, accordion_button};
use crate::app::widgets::toast::Toast;
use crate::{fl, icons};

pub struct UpdateApplications {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,

    /// Callback after loading the applications list
    LoadedApplicationsList(Result<Vec<Application>, anywho::Error>),
    /// Refreshes the applications list
    RefreshApplicationsList,

    /// Attempt to update all applications
    UpdateAllApplications,
    /// Callback after updating all applications
    UpdatedAllApplications(Result<Vec<UpdateResult>, UpdateError>),

    /// Attempt to update a single application
    UpdateSingleApplication(String),
    /// Callback after updating a single application
    UpdatedSingleApplication(Result<(), UpdateError>),
}

pub enum State {
    Loading,
    Ready { applications: Vec<Application> },
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl UpdateApplications {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                Application::get_all_available_updates(),
                Message::LoadedApplicationsList,
            ),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let State::Ready { applications, .. } = &self.state else {
            return container(text(fl!("checking-updates")))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        };

        let any_app_updating = applications
            .iter()
            .any(|app| app.application_status == ApplicationStatus::Updating);

        let content = if applications.is_empty() {
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
        } else {
            let mut applications_accordion = column![]
                .align_x(Alignment::Center)
                .height(Length::Fill)
                .width(Length::Fill);

            for (index, app) in applications.iter().enumerate() {
                let button_status =
                    if app.application_status == ApplicationStatus::Updating || any_app_updating {
                        AccordionButtonStatus::Disabled
                    } else {
                        AccordionButtonStatus::Enabled
                    };

                let icon = if let Some(svg_path) = &app.icon {
                    AccordionIcon::Path {
                        svg_path: svg_path.to_string(),
                    }
                } else {
                    AccordionIcon::None
                };

                if index == 0 {
                    applications_accordion = applications_accordion.push(accordion_button(
                        AccordionButtonPosition::Top,
                        app.name.to_string(),
                        format!(
                            "{} >>> {} (Hash may be different)",
                            app.current_version, app.latest_version
                        ),
                        icon,
                        Message::UpdateSingleApplication(app.app_id.clone()),
                        button_status,
                    ));
                } else if index == applications.len() {
                    applications_accordion = applications_accordion.push(accordion_button(
                        AccordionButtonPosition::Bottom,
                        app.name.to_string(),
                        format!(
                            "{} >>> {} (Hash may be different)",
                            app.current_version, app.latest_version
                        ),
                        icon,
                        Message::UpdateSingleApplication(app.app_id.clone()),
                        button_status,
                    ));
                } else {
                    applications_accordion = applications_accordion.push(accordion_button(
                        AccordionButtonPosition::Middle,
                        app.name.to_string(),
                        format!(
                            "{} >>> {} (Hash may be different)",
                            app.current_version, app.latest_version
                        ),
                        icon,
                        Message::UpdateSingleApplication(app.app_id.clone()),
                        button_status,
                    ));
                }
            }

            column![
                Space::new(Length::Fill, Length::Fixed(35.)),
                row![
                    text(fl!("application-updates"))
                        .width(Length::Fill)
                        .size(18)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),
                    button(text(fl!("update-all")))
                        .style(primary_button_style)
                        .on_press_maybe(if any_app_updating {
                            None
                        } else {
                            Some(Message::UpdateAllApplications)
                        })
                ],
                scrollable(applications_accordion),
            ]
            .padding(20.)
            .spacing(5.)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Alignment::Center)
        };

        let main_content = container(content)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let back_button = container(
            button(icons::get_icon("go-previous-symbolic", 18))
                .on_press_maybe(if any_app_updating {
                    None
                } else {
                    Some(Message::Back)
                })
                .style(icon_button_style),
        )
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        let refresh_button = container(
            button(icons::get_icon("view-refresh-symbolic", 18))
                .on_press_maybe(if any_app_updating {
                    None
                } else {
                    Some(Message::RefreshApplicationsList)
                })
                .style(icon_button_style),
        )
        .align_x(Alignment::End)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        iced::widget::stack![main_content, back_button, refresh_button].into()
    }

    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
            Message::LoadedApplicationsList(result) => match result {
                Ok(applications) => {
                    self.state = State::Ready { applications };
                    Action::None
                }
                Err(err) => {
                    self.state = State::Ready {
                        applications: Vec::new(),
                    };
                    Action::AddToast(Toast::error_toast(err.to_string()))
                }
            },
            Message::RefreshApplicationsList => {
                self.state = State::Loading;
                Action::Run(Task::perform(
                    Application::get_all_available_updates(),
                    Message::LoadedApplicationsList,
                ))
            }
            Message::UpdateAllApplications => {
                let State::Ready { applications, .. } = &mut self.state else {
                    return Action::None;
                };

                if applications.is_empty() {
                    Action::None
                } else {
                    applications
                        .iter_mut()
                        .for_each(|a| a.application_status = ApplicationStatus::Updating);

                    Action::Run(Task::perform(
                        Application::update_all(applications.to_vec()),
                        Message::UpdatedAllApplications,
                    ))
                }
            }
            Message::UpdatedAllApplications(update_results) => {
                // TODO
                if let Err(err) = update_results {
                    return Action::AddToast(Toast::error_toast(err));
                }

                self.state = State::Loading;
                Action::Run(Task::perform(
                    Application::get_all_available_updates(),
                    Message::LoadedApplicationsList,
                ))
            }
            Message::UpdateSingleApplication(application_id) => {
                let State::Ready { applications, .. } = &mut self.state else {
                    return Action::None;
                };

                if applications.is_empty() {
                    Action::None
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(application) =
                        applications.iter_mut().find(|a| a.app_id == application_id)
                    {
                        application.application_status = ApplicationStatus::Updating;
                        Action::Run(Task::perform(
                            Application::update(application.app_id.clone()),
                            Message::UpdatedSingleApplication,
                        ))
                    } else {
                        Action::AddToast(Toast::error_toast("Application not found!"))
                    }
                }
            }
            Message::UpdatedSingleApplication(result) => {
                // TODO
                if let Err(err) = result {
                    return Action::AddToast(Toast::error_toast(err));
                }

                self.state = State::Loading;
                Action::Run(Task::perform(
                    Application::get_all_available_updates(),
                    Message::LoadedApplicationsList,
                ))
            }
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
