use iced::time::Instant;
use iced::widget::text::LineHeight;
use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};
use iced::{Padding, Task};

use crate::app::core::layered_packages::{
    CheckPackageError, add_packages, check_package, remove_packages,
};
use crate::app::style::{
    TabButtonPosition, danger_icon_button_style, icon_button_style, icon_svg_style,
    primary_button_style, rounded_button_combo_style, rounded_input_combo_style,
    rounderer_box_container_style, tab_button_style,
};
use crate::app::widgets::spinners::circular::Circular;
use crate::app::widgets::spinners::easing;
use crate::app::{core::system_status::Deployment, widgets::toast::Toast};
use crate::{fl, icons};

pub struct LayeredPackages {
    applying_changes: bool,
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
    /// Toggles the package removal state
    TogglePackageToRemove(String, bool),

    /// Attempts to apply the current changes, changes to apply vary depending on the current tab the user is on
    ApplyChanges,
    /// Callback after attempting to apply the current changes
    ApplyChangesCallback(Result<(), anywho::Error>),
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
    BackAndCheckReboot,
}

/// Represents each possible open Tab of the LayeredPackages Subscreen
#[derive(Debug)]
enum Tab {
    AddPackages {
        package_name_input: String,
        packages_to_add: Vec<String>,
    },
    RemovePackages {
        packages_to_remove: Vec<(String, bool)>,
    },
}

impl Tab {
    fn kind(&self) -> &'static str {
        match self {
            Tab::AddPackages { .. } => "AddPackages",
            Tab::RemovePackages { .. } => "RemovePackages",
        }
    }

    pub fn is_add_packages(&self) -> bool {
        matches!(self, Tab::AddPackages { .. })
    }

    pub fn is_remove_packages(&self) -> bool {
        matches!(self, Tab::RemovePackages { .. })
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self::AddPackages {
            package_name_input: String::new(),
            packages_to_add: Vec::new(),
        }
    }
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
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
                applying_changes: false,
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
                    if !packages_to_remove.iter().any(|x| x.1) {
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
                        let packages_to_remove: Vec<(String, bool)> = self
                            .current_packages
                            .iter()
                            .cloned()
                            .map(|pkg| (pkg, false))
                            .collect();

                        self.current_tab = Tab::RemovePackages { packages_to_remove };
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
            Message::TogglePackageToRemove(package_name, new_value) => {
                if let Tab::RemovePackages {
                    packages_to_remove, ..
                } = &mut self.current_tab
                {
                    packages_to_remove.iter_mut().for_each(|p| {
                        if p.0 == package_name {
                            p.1 = new_value;
                        }
                    });
                }
                Action::None
            }
            Message::ApplyChanges => match &self.current_tab {
                Tab::AddPackages {
                    packages_to_add, ..
                } => {
                    if !packages_to_add.is_empty() {
                        self.applying_changes = true;
                        Action::Run(Task::perform(
                            add_packages(packages_to_add.clone()),
                            Message::ApplyChangesCallback,
                        ))
                    } else {
                        Action::AddToast(Toast::warning_toast("No packages to add"))
                    }
                }
                Tab::RemovePackages { packages_to_remove } => {
                    let packages_marked_to_remove: Vec<String> = packages_to_remove
                        .iter()
                        .filter(|x| x.1)
                        .map(|x| x.0.clone())
                        .collect();
                    if !packages_marked_to_remove.is_empty() {
                        self.applying_changes = true;
                        Action::Run(Task::perform(
                            remove_packages(packages_marked_to_remove.clone()),
                            Message::ApplyChangesCallback,
                        ))
                    } else {
                        Action::AddToast(Toast::warning_toast("No packages to remove"))
                    }
                }
            },
            Message::ApplyChangesCallback(result) => match result {
                Ok(_) => Action::BackAndCheckReboot,
                Err(err) => {
                    self.applying_changes = false;
                    Action::AddToast(Toast::error_toast(err))
                }
            },
        }
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        if self.applying_changes {
            return container(
                column![
                    text(fl!("applying-changes")),
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
                    .width(Length::Fill)
                    .style(rounded_input_combo_style)
                    .line_height(LineHeight::Relative(2.)),
                    button(text(fl!("add")).line_height(LineHeight::Relative(2.)))
                        .style(primary_button_style)
                        .on_press(Message::CheckPackageBeforeAddList)
                        .style(rounded_button_combo_style)
                ];

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
                                        container(
                                            text(package)
                                                .font(iced::font::Font {
                                                    weight: iced::font::Weight::Bold,
                                                    ..Default::default()
                                                })
                                                .width(Length::Fill)
                                        )
                                        .padding(Padding::new(0.).left(10.))
                                        .width(Length::Fill),
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
                                .style(rounderer_box_container_style)
                                .align_y(Alignment::Center)
                                .width(Length::Fill)
                                .padding(10.),
                            )
                        })
                        .push(
                            container(text(format!(
                                "{} {}",
                                packages_to_add.len(),
                                fl!("packages-to-add-footer")
                            )))
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                            .padding(10.),
                        )
                        .into()
                };

                column![search_package_input_row, packages_to_add_content].spacing(5.)
            }
            Tab::RemovePackages { packages_to_remove } => packages_to_remove
                .iter()
                .fold(
                    column![].spacing(5.).width(Length::Fill),
                    |col, (package_name, select_status)| {
                        col.push(
                            container(
                                row![
                                    container(
                                        text(package_name)
                                            .font(iced::font::Font {
                                                weight: iced::font::Weight::Bold,
                                                ..Default::default()
                                            })
                                            .width(Length::Fill)
                                    )
                                    .padding(Padding::new(0.).left(10.))
                                    .width(Length::Fill),
                                    checkbox(fl!("remove"), *select_status)
                                        .on_toggle(|new_value| {
                                            Message::TogglePackageToRemove(
                                                package_name.to_string(),
                                                new_value,
                                            )
                                        })
                                        .style(checkbox::danger),
                                ]
                                .align_y(Alignment::Center)
                                .width(Length::Fill),
                            )
                            .style(rounderer_box_container_style)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .padding(10.),
                        )
                    },
                )
                .push(
                    container(text(format!(
                        "{} {}",
                        packages_to_remove.iter().filter(|x| x.1).count(),
                        fl!("packages-to-remove-footer")
                    )))
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(10.),
                ),
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
                button(text(fl!("apply-changes")))
                    .on_press(Message::ApplyChanges)
                    .style(primary_button_style)
            ],
            row![
                button(
                    text(fl!("add-packages"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(16)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                )
                .on_press(Message::OpenAddPackagesTab)
                .style(|t, s| tab_button_style(
                    t,
                    s,
                    self.current_tab.is_add_packages(),
                    TabButtonPosition::Left
                ))
                .width(Length::Fill),
                button(
                    text(fl!("remove-packages"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(16)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                )
                .on_press(Message::OpenRemovePackagesTab)
                .style(|t, s| tab_button_style(
                    t,
                    s,
                    self.current_tab.is_remove_packages(),
                    TabButtonPosition::Right
                ))
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
