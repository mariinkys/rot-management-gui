// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

use app::FAManagement;
use iced::{
    Size,
    window::{Settings, icon},
};

mod app;
mod i18n;
mod icons;

fn main() -> iced::Result {
    // Init the icon cache
    icons::ICON_CACHE.get_or_init(|| std::sync::Mutex::new(icons::IconCache::new()));

    // Get the window  icon
    let icon = icon::from_file_data(
        include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg"),
        None,
    );

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    iced::application::timed(
        FAManagement::new,
        FAManagement::update,
        FAManagement::subscription,
        FAManagement::view,
    )
    .window(Settings {
        position: iced::window::Position::Centered,
        icon: icon.ok(),
        resizable: true,
        size: Size::new(800., 600.),
        ..Default::default()
    })
    .title("RPM OSTree Management GUI")
    .theme(FAManagement::theme)
    .run()
}
