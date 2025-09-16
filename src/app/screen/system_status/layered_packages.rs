use iced::Task;
use iced::time::Instant;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::core::layered_packages::{CheckPackageError, check_package};
use crate::app::style::{
    danger_icon_button_style, icon_button_style, icon_svg_style, primary_button_style,
};
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

    /// Callback when inputting text on Add Package Input
    AddPackageInputUpdated(String),
    /// Check if the package exists before adding it to the add list
    CheckPackageBeforeAddList,
    /// Callback after checking for the package
    PackageToAddChecked(Result<String, CheckPackageError>),
    /// Removes a package from the packages to add list
    RemovePackageFromAddList(String),
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
            Message::AddPackageInputUpdated(new_value) => {
                if let Tab::AddPackages {
                    package_name_input, ..
                } = &mut self.current_tab
                {
                    *package_name_input = new_value;
                }

                Action::None
            }
            Message::CheckPackageBeforeAddList => {
                if let Tab::AddPackages {
                    package_name_input,
                    packages_to_add,
                    ..
                } = &self.current_tab
                {
                    let trimmed_package_name = package_name_input.trim().to_string();

                    if packages_to_add.contains(&trimmed_package_name) {
                        return Action::AddToast(Toast::warning_toast(
                            "This package is already on the add list",
                        ));
                    }

                    if self.current_packages.contains(&trimmed_package_name) {
                        return Action::AddToast(Toast::warning_toast("Package already installed"));
                    }

                    if !trimmed_package_name.is_empty() {
                        return Action::Run(Task::perform(
                            check_package(trimmed_package_name),
                            Message::PackageToAddChecked,
                        ));
                    }
                }

                Action::None
            }
            Message::PackageToAddChecked(result) => match result {
                Ok(package_name) => {
                    if let Tab::AddPackages {
                        package_name_input,
                        packages_to_add,
                    } = &mut self.current_tab
                    {
                        *package_name_input = String::new();
                        packages_to_add.push(package_name);
                    }
                    Action::None
                }
                Err(err) => match err {
                    CheckPackageError::NotFound => {
                        Action::AddToast(Toast::warning_toast("Package not found"))
                    }
                    CheckPackageError::Error(error) => Action::AddToast(Toast::error_toast(error)),
                },
            },
            Message::RemovePackageFromAddList(package_name) => {
                if let Tab::AddPackages {
                    packages_to_add, ..
                } = &mut self.current_tab
                {
                    packages_to_add.retain(|n| *n != package_name);
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
            } => {
                let search_package_input_row = row![
                    text_input(
                        fl!("search-package-add-placeholder").as_str(),
                        package_name_input,
                    )
                    .on_input(Message::AddPackageInputUpdated)
                    .on_paste(Message::AddPackageInputUpdated)
                    .on_submit(Message::CheckPackageBeforeAddList)
                    .width(Length::Fill),
                    button(text(fl!("add")))
                        .style(primary_button_style)
                        .on_press(Message::CheckPackageBeforeAddList)
                ]
                .spacing(3.);

                let packages_to_add_content: Element<Message> = if packages_to_add.is_empty() {
                    container(text(fl!("no-packages-add-list")))
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                        .padding(5.)
                        .into()
                } else {
                    packages_to_add
                        .iter()
                        .fold(column![].spacing(5.).width(Length::Fill), |col, package| {
                            col.push(
                                container(
                                    row![
                                        text(package).width(Length::Fill),
                                        button(
                                            icons::get_icon("user-trash-full-symbolic", 18)
                                                .style(icon_svg_style)
                                        )
                                        .on_press(Message::RemovePackageFromAddList(
                                            package.to_string()
                                        ))
                                        .style(danger_icon_button_style),
                                    ]
                                    .align_y(Alignment::Center)
                                    .width(Length::Fill),
                                )
                                .style(container::rounded_box)
                                .align_y(Alignment::Center)
                                .width(Length::Fill)
                                .padding(10.),
                            )
                        })
                        .into()
                };

                column![search_package_input_row, packages_to_add_content].spacing(5.)
            }
            Tab::RemovePackages { packages_to_remove } => todo!(),
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
