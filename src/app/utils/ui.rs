// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{
        style::{AccordionButtonPosition, accordion_button_style, icon_svg_style},
        widgets::spinners::{circular::Circular, easing},
    },
    icons,
};
use iced::{
    Alignment, Length, Renderer, Theme,
    widget::{Button, Space, button, column, container, image, row, svg, text},
};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum AccordionButtonStatus {
    Disabled,
    Enabled,
}

pub enum PossibleBundledSVGs {
    UpdateSystem,
    UpdateApplications,
    SystemStatus,
    Rollback,
}

impl PossibleBundledSVGs {
    pub fn get(&self) -> iced::widget::svg::Handle {
        match &self {
            PossibleBundledSVGs::UpdateSystem => {
                icons::get_handle("system-software-install-symbolic", 18)
            }
            PossibleBundledSVGs::UpdateApplications => {
                icons::get_handle("software-update-available-symbolic", 18)
            }
            PossibleBundledSVGs::SystemStatus => {
                icons::get_handle("utilities-system-monitor-symbolic", 18)
            }
            PossibleBundledSVGs::Rollback => icons::get_handle("view-refresh-symbolic", 18),
        }
    }
}

pub enum AccordionIcon {
    None,
    SvgPath { svg_path: String },
    ImgPath { img_path: String },
    BundledSvg { svg: PossibleBundledSVGs },
}

pub fn accordion_button<'a, Message>(
    button_position: AccordionButtonPosition,
    main_text: String,
    description: String,
    icon: AccordionIcon,
    message: Message,
    button_status: AccordionButtonStatus,
    show_spinner: bool,
) -> Button<'a, Message, Theme, Renderer>
where
    Message: Clone + Debug + 'a,
{
    let on_press = if button_status == AccordionButtonStatus::Disabled {
        None
    } else {
        Some(message)
    };

    let icon: iced::Element<'_, Message> = match icon {
        AccordionIcon::None => Space::new(0, 0).into(),
        AccordionIcon::SvgPath { svg_path } => {
            let handle = svg::Handle::from_path(svg_path);
            container(
                svg::Svg::new(handle)
                    .width(iced::Length::Fixed(50.))
                    .height(iced::Length::Fixed(50.)),
            )
            .width(Length::Fixed(50.))
            .height(Length::Fixed(50.))
            .align_y(Alignment::Center)
            .into()
        }
        AccordionIcon::ImgPath { img_path } => {
            let handle = image::Handle::from_path(img_path);
            container(
                image::Image::new(handle)
                    .width(iced::Length::Fixed(50.))
                    .height(iced::Length::Fixed(50.))
                    .content_fit(iced::ContentFit::Contain),
            )
            .width(Length::Fixed(50.))
            .height(Length::Fixed(50.))
            .align_y(Alignment::Center)
            .into()
        }
        AccordionIcon::BundledSvg { svg } => {
            let handle = svg.get();
            container(
                svg::Svg::new(handle)
                    .width(iced::Length::Fixed(40.))
                    .height(iced::Length::Fixed(40.))
                    .style(icon_svg_style),
            )
            .style(container::transparent)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(40.))
            .align_y(Alignment::Center)
            .into()
        }
    };

    let spinner: iced::Element<'_, Message> = if show_spinner {
        Circular::new()
            .easing(&easing::EMPHASIZED)
            .cycle_duration(std::time::Duration::from_secs_f32(5.0))
            .into()
    } else {
        Space::new(0, 0).into()
    };

    button(
        row![
            icon,
            container(
                column![
                    text(main_text)
                        .align_y(Alignment::Center)
                        .size(18)
                        .style(text::base)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),
                    text(description)
                        .align_y(Alignment::Center)
                        .size(13)
                        .style(text::base)
                ]
                .spacing(2.)
                .height(Length::Shrink)
                .width(Length::Fill)
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .align_y(Alignment::Center),
            spinner,
        ]
        .spacing(10.)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .style(move |theme: &Theme, status: button::Status| {
        accordion_button_style(theme, status, &button_position)
    })
    .width(Length::Fill)
    .height(80.)
    .on_press_maybe(on_press)
}
