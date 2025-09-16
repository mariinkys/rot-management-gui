use iced::Task;
use iced::time::Instant;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::style::{icon_button_style, icon_svg_style, primary_button_style};
use crate::app::{core::system_status::Deployment, widgets::toast::Toast};
use crate::{fl, icons};

pub struct LayeredPackages {
    current_packages: Vec<String>,
    current_tab: Tab,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to go back a screen                     
    Back,

    /// Call to switch to the AddPackages Tab
    OpenAddPackagesTab,
    /// Call to switch to the RemovePackages Tab
    OpenRemovePackagesTab,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

/// Represents each possible open Tab of the LayeredPackages Subscreen
#[derive(Debug)]
enum Tab {
    AddPackages {
        package_name_input: String,
        packages_to_add: Vec<String>,
    },
    RemovePackages {
        packages_to_remove: Vec<String>,
    },
}

impl Default for Tab {
    fn default() -> Self {
        Self::AddPackages {
            package_name_input: String::new(),
            packages_to_add: Vec::new(),
        }
    }
}

impl LayeredPackages {
    pub fn new(current_deployment: Deployment) -> (Self, Task<Message>) {
        let current_packages: Vec<String> = current_deployment
            .layered_packages
            .split(' ')
            .map(|s| s.trim().to_string())
            .collect();

        (
            Self {
                current_packages,
                current_tab: Tab::default(),
            },
            Task::none(),
        )
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Back => Action::Back,
            Message::OpenAddPackagesTab => {
                if let Tab::RemovePackages { packages_to_remove } = &self.current_tab {
                    if packages_to_remove.is_empty() {
                        self.current_tab = Tab::AddPackages {
                            package_name_input: String::new(),
                            packages_to_add: Vec::new(),
                        };
                    } else {
                        return Action::AddToast(Toast::warning_toast(
                            "You have packages to remove selected, changes will be lost if you change Tab's",
                        ));
                    }
                }

                Action::None
            }
            Message::OpenRemovePackagesTab => {
                if self.current_packages.is_empty() {
                    return Action::AddToast(Toast::warning_toast(
                        "You have no packages available to remove",
                    ));
                }

                if let Tab::AddPackages {
                    packages_to_add, ..
                } = &self.current_tab
                {
                    if packages_to_add.is_empty() {
                        self.current_tab = Tab::RemovePackages {
                            packages_to_remove: Vec::new(),
                        };
                    } else {
                        return Action::AddToast(Toast::warning_toast(
                            "You have packages to add selected, changes will be lost if you change Tab's",
                        ));
                    }
                }

                Action::None
            }
        }
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        let tab_content = match &self.current_tab {
            Tab::AddPackages {
                package_name_input,
                packages_to_add,
            } => text("Add Packages Tab Content"),
            Tab::RemovePackages { packages_to_remove } => text("Remove Packages Tab Content"),
        };

        let content: Element<Message> = column![
            Space::new(Length::Fill, Length::Fixed(35.)),
            row![
                text(fl!("manage-layered-packages"))
                    .width(Length::Fill)
                    .size(18)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                button(text(fl!("apply-changes"))).style(primary_button_style)
            ],
            row![
                button(
                    text(fl!("add-packages"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                )
                .on_press(Message::OpenAddPackagesTab)
                .style(button::subtle)
                .width(Length::Fill),
                button(
                    text(fl!("remove-packages"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                )
                .on_press(Message::OpenRemovePackagesTab)
                .style(button::subtle)
                .width(Length::Fill)
            ]
            .width(Length::Fill)
            .align_y(Alignment::Center),
            tab_content
        ]
        .padding(20.)
        .spacing(5.)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into();

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
