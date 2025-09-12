// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::{button, center, column, container};
use iced::{Alignment, Length, Subscription, Theme};
use iced::{Task, widget::text};

use crate::app::core::config::Config;
use crate::app::screen::{
    Screen, about, config, rollback, system_status, update_applications, update_system,
};
use crate::app::style::{AccordionButtonPosition, icon_button_style};
use crate::app::utils::ui::{
    AccordionButtonStatus, AccordionIcon, PossibleBundledSVGs, accordion_button,
};
use crate::app::widgets::toast::{self, Toast};
use crate::{fl, icons};

pub mod core;
pub mod screen;
pub mod style;
pub mod utils;
pub mod widgets;

const APP_ID: &str = "dev.mariinkys.FaManagementGUI";

pub struct FAManagement {
    toasts: Vec<Toast>,
    state: State,
    now: Instant,
}

enum State {
    Loading,
    Ready { config: Config, screen: Screen },
}

#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(Result<Config, anywho::Error>),
    ConfigSaved(Result<(), anywho::Error>),

    UpdateSystem(update_system::Message),
    UpdateApplications(update_applications::Message),
    Rollback(rollback::Message),
    SystemStatus(system_status::Message),
    Config(config::Message),
    About(about::Message),

    OpenUpdateSystem,
    OpenUpdateApplications,
    OpenRollback,
    OpenSystemStatus,
    OpenConfig,
    OpenAbout,

    AddToast(Toast),
    CloseToast(usize),
}

impl FAManagement {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                toasts: Vec::new(),
                state: State::Loading,
                now: Instant::now(),
            },
            Task::perform(
                async move { Config::load(APP_ID).await },
                Message::ConfigLoaded,
            ),
        )
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let content = match &self.state {
            State::Loading => center(text("Loading...")).into(),
            State::Ready { screen, .. } => match screen {
                Screen::Welcome => self.welcome_view(),
                Screen::UpdateSystem(update_system) => {
                    update_system.view(self.now).map(Message::UpdateSystem)
                }
                Screen::UpdateApplications(update_applications) => update_applications
                    .view(self.now)
                    .map(Message::UpdateApplications),
                Screen::Rollback(rollback) => rollback.view(self.now).map(Message::Rollback),
                Screen::SystemStatus(system_status) => {
                    system_status.view(self.now).map(Message::SystemStatus)
                }
                Screen::Config(config) => config.view(self.now).map(Message::Config),
                Screen::About(about) => about.view(self.now).map(Message::About),
            },
        };

        toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::ConfigLoaded(res) => {
                match res {
                    Ok(config) => {
                        self.state = State::Ready {
                            config,
                            screen: Screen::Welcome,
                        };
                    }
                    Err(err) => {
                        eprintln!("Error loading config: {err}");
                    }
                }
                return Task::none();
            }
            Message::ConfigSaved(res) => {
                if let Err(err) = res {
                    eprintln!("{err}");
                }
                return Task::none();
            }

            Message::UpdateSystem(message) => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::UpdateSystem(update_system) = screen else {
                    return Task::none();
                };

                return match update_system.update(message, self.now) {
                    update_system::Action::None => Task::none(),
                    update_system::Action::Run(task) => task.map(Message::UpdateSystem),
                    update_system::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    update_system::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                    update_system::Action::AddToastAndRun((toast, task)) => {
                        self.toasts.push(toast);
                        task.map(Message::UpdateSystem)
                    }
                };
            }
            Message::OpenUpdateSystem => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let (update_system, task) = screen::UpdateSystem::new();
                *screen = Screen::UpdateSystem(update_system);
                return task.map(Message::UpdateSystem);
            }

            Message::UpdateApplications(message) => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::UpdateApplications(update_applications) = screen else {
                    return Task::none();
                };

                return match update_applications.update(message, self.now) {
                    update_applications::Action::None => Task::none(),
                    update_applications::Action::Run(task) => task.map(Message::UpdateApplications),
                    update_applications::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    update_applications::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                    update_applications::Action::AddMultipleToasts(toasts) => {
                        for toast in toasts {
                            self.toasts.push(toast);
                        }
                        return Task::none();
                    }
                };
            }
            Message::OpenUpdateApplications => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let (update_applications, task) = screen::UpdateApplications::new();
                *screen = Screen::UpdateApplications(update_applications);
                return task.map(Message::UpdateApplications);
            }

            Message::Rollback(message) => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::Rollback(rollback) = screen else {
                    return Task::none();
                };

                return match rollback.update(message, self.now) {
                    rollback::Action::None => Task::none(),
                    rollback::Action::Run(task) => task.map(Message::Rollback),
                    rollback::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    rollback::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                    rollback::Action::AddToastAndRun((toast, task)) => {
                        self.toasts.push(toast);
                        task.map(Message::Rollback)
                    }
                };
            }
            Message::OpenRollback => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let (rollback, task) = screen::Rollback::new();
                *screen = Screen::Rollback(rollback);
                return task.map(Message::Rollback);
            }

            Message::SystemStatus(message) => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::SystemStatus(system_status) = screen else {
                    return Task::none();
                };

                return match system_status.update(message, self.now) {
                    system_status::Action::None => Task::none(),
                    system_status::Action::Run(task) => task.map(Message::SystemStatus),
                    system_status::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    system_status::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                    system_status::Action::AddToastAndRun((toast, task)) => {
                        self.toasts.push(toast);
                        task.map(Message::SystemStatus)
                    }
                };
            }
            Message::OpenSystemStatus => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let (system_status, task) = screen::SystemStatus::new();
                *screen = Screen::SystemStatus(system_status);
                return task.map(Message::SystemStatus);
            }

            Message::Config(message) => {
                let State::Ready { screen, config, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::Config(config_screen) = screen else {
                    return Task::none();
                };

                return match config_screen.update(message, self.now) {
                    config::Action::ChangedTheme(application_theme) => {
                        config.theme = application_theme;
                        let new_config = config.clone();
                        Task::perform(new_config.save(APP_ID), Message::ConfigSaved)
                    }
                    config::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                };
            }
            Message::OpenConfig => {
                let State::Ready { screen, config, .. } = &mut self.state else {
                    return Task::none();
                };

                let (config, task) = screen::Config::new(config.clone());
                *screen = Screen::Config(config);
                return task.map(Message::Config);
            }
            Message::About(message) => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let Screen::About(about_screen) = screen else {
                    return Task::none();
                };

                match about_screen.update(message, self.now) {
                    about::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    about::Action::None => return Task::none(),
                }
            }
            Message::OpenAbout => {
                let State::Ready { screen, .. } = &mut self.state else {
                    return Task::none();
                };

                let (about, task) = screen::About::new();
                *screen = Screen::About(about);
                return task.map(Message::About);
            }

            Message::AddToast(toast) => {
                self.toasts.push(toast);
            }
            Message::CloseToast(index) => {
                self.toasts.remove(index);
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let State::Ready { screen, .. } = &self.state else {
            return Subscription::none();
        };

        match screen {
            Screen::Welcome => Subscription::none(),
            Screen::UpdateSystem(update_system) => update_system
                .subscription(self.now)
                .map(Message::UpdateSystem),
            Screen::UpdateApplications(update_applications) => update_applications
                .subscription(self.now)
                .map(Message::UpdateApplications),
            Screen::Rollback(rollback) => rollback.subscription(self.now).map(Message::Rollback),
            Screen::SystemStatus(system_status) => system_status
                .subscription(self.now)
                .map(Message::SystemStatus),
            Screen::Config(config) => config.subscription(self.now).map(Message::Config),
            Screen::About(about) => about.subscription(self.now).map(Message::About),
        }
    }

    pub fn theme(&self) -> Theme {
        let State::Ready { config, .. } = &self.state else {
            return iced::Theme::Light;
        };

        config.theme.clone().into()
    }

    fn welcome_view(&self) -> iced::Element<'_, Message> {
        let welcome_text = text(fl!("welcome")).size(32).font(iced::font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        });
        let subtitle = text(fl!("subtitle")).size(24);
        let top_text = column![welcome_text, subtitle]
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .spacing(3.);

        let page_buttons = column![
            accordion_button(
                AccordionButtonPosition::Top,
                fl!("update-system"),
                fl!("update-system-description"),
                AccordionIcon::BundledSvg {
                    svg: PossibleBundledSVGs::UpdateSystem
                },
                Message::OpenUpdateSystem,
                AccordionButtonStatus::Enabled,
            ),
            accordion_button(
                AccordionButtonPosition::Middle,
                fl!("update-applications"),
                fl!("update-applications-description"),
                AccordionIcon::BundledSvg {
                    svg: PossibleBundledSVGs::UpdateApplications
                },
                Message::OpenUpdateApplications,
                AccordionButtonStatus::Enabled,
            ),
            accordion_button(
                AccordionButtonPosition::Middle,
                fl!("rollback"),
                fl!("rollback-description"),
                AccordionIcon::BundledSvg {
                    svg: PossibleBundledSVGs::Rollback
                },
                Message::OpenRollback,
                AccordionButtonStatus::Enabled,
            ),
            accordion_button(
                AccordionButtonPosition::Bottom,
                fl!("system-status"),
                fl!("system-status-description"),
                AccordionIcon::BundledSvg {
                    svg: PossibleBundledSVGs::SystemStatus
                },
                Message::OpenSystemStatus,
                AccordionButtonStatus::Enabled,
            )
        ]
        .spacing(0.)
        .height(Length::Shrink);

        let main_content = column![top_text, page_buttons]
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Center)
            .padding(20.)
            .spacing(30.);

        let centered_content = container(main_content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let config_button = container(
            button(icons::get_icon("emblem-system-symbolic", 18))
                .on_press(Message::OpenConfig)
                .style(icon_button_style),
        )
        .align_x(Alignment::End)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        let about_button = container(
            button(icons::get_icon("help-info-symbolic", 18))
                .on_press(Message::OpenAbout)
                .style(icon_button_style),
        )
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.);

        iced::widget::stack![centered_content, config_button, about_button].into()
    }
}
