// SPDX-License-Identifier: GPL-3.0-only

use iced::{
    Theme,
    widget::{button, container, text_input},
};

pub fn icon_svg_style(
    theme: &Theme,
    _status: iced::widget::svg::Status,
) -> iced::widget::svg::Style {
    iced::widget::svg::Style {
        color: Some(theme.palette().text),
    }
}

pub fn icon_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::subtle(theme, status);
    style.border.radius = iced::border::Radius {
        top_left: 25.0,
        top_right: 25.0,
        bottom_left: 25.0,
        bottom_right: 25.0,
    };
    style
}

pub fn rounded_input_combo_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border.radius = iced::border::Radius {
        top_left: 25.0,
        top_right: 0.0,
        bottom_left: 25.0,
        bottom_right: 0.0,
    };
    style
}

pub fn rounded_button_combo_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::success(theme, status);
    style.border.radius = iced::border::Radius {
        top_left: 0.0,
        top_right: 25.0,
        bottom_left: 0.0,
        bottom_right: 25.0,
    };
    style
}

pub enum TabButtonPosition {
    Left,
    #[allow(dead_code)]
    Middle,
    Right,
}

pub fn tab_button_style(
    theme: &Theme,
    status: button::Status,
    current_tab: bool,
    button_position: TabButtonPosition,
) -> button::Style {
    let mut style = button::subtle(theme, status);

    if current_tab {
        style.background = Some(iced::Background::Color(
            theme.extended_palette().background.strongest.color,
        ));
    }

    match button_position {
        TabButtonPosition::Left => {
            style.border.radius = iced::border::Radius {
                top_left: 25.0,
                top_right: 0.0,
                bottom_left: 25.0,
                bottom_right: 0.0,
            };
        }
        TabButtonPosition::Middle => {}
        TabButtonPosition::Right => {
            style.border.radius = iced::border::Radius {
                top_left: 0.0,
                top_right: 25.0,
                bottom_left: 0.0,
                bottom_right: 25.0,
            };
        }
    }

    style
}

pub fn danger_icon_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = iced::border::Radius {
        top_left: 25.0,
        top_right: 25.0,
        bottom_left: 25.0,
        bottom_right: 25.0,
    };
    style
}

pub fn rounderer_box_container_style(theme: &Theme) -> container::Style {
    let mut style = container::rounded_box(theme);
    style.border.radius = iced::border::Radius {
        top_left: 25.0,
        top_right: 25.0,
        bottom_left: 25.0,
        bottom_right: 25.0,
    };
    style
}

pub enum AccordionButtonPosition {
    Top,
    Middle,
    Bottom,
}

pub fn accordion_button_style(
    theme: &Theme,
    status: button::Status,
    button_position: &AccordionButtonPosition,
) -> button::Style {
    let mut style = button::primary(theme, status);

    style.background = match status {
        button::Status::Active => Some(iced::Background::Color(theme.palette().background)),
        button::Status::Hovered => Some(iced::Background::Color(iced::Color {
            r: theme.palette().background.r * 0.85,
            g: theme.palette().background.g * 0.85,
            b: theme.palette().background.b * 0.85,
            a: theme.palette().background.a,
        })),
        button::Status::Pressed => Some(iced::Background::Color(iced::Color {
            r: theme.palette().background.r * 0.7,
            g: theme.palette().background.g * 0.7,
            b: theme.palette().background.b * 0.7,
            a: theme.palette().background.a,
        })),
        button::Status::Disabled => Some(iced::Background::Color(iced::Color {
            r: theme.palette().background.r * 0.6,
            g: theme.palette().background.g * 0.6,
            b: theme.palette().background.b * 0.6,
            a: 0.5,
        })),
    };

    match button_position {
        AccordionButtonPosition::Top => {
            style.border.radius = iced::border::Radius {
                top_left: 5.0,
                top_right: 5.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            };
            style.border.width = 1.0;
            style.border.color = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3);
        }
        AccordionButtonPosition::Middle => {
            style.border.radius = iced::border::Radius::default();
            style.border.width = 1.0;
            style.border.color = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3);
        }
        AccordionButtonPosition::Bottom => {
            style.border.radius = iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 5.0,
                bottom_right: 5.0,
            };
            style.border.width = 1.0;
            style.border.color = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3);
        }
    };

    style
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = iced::border::Radius {
        top_left: 15.0,
        top_right: 15.0,
        bottom_left: 15.0,
        bottom_right: 15.0,
    };
    style
}
