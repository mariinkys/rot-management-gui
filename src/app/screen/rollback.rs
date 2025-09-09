// SPDX-License-Identifier: GPL-3.0-only

use iced::time::Instant;
use iced::widget::text;
use iced::{Subscription, Task};

use crate::app::widgets::toast::Toast;

pub struct Rollback {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks to add a toast to the parent state
    AddToast(Toast),
    /// Asks to go back a screen                     
    Back,
}

pub enum State {
    Loading,
    Ready {},
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Rollback {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Ready {},
            },
            Task::none(),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        text("Update System").into()
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update(&mut self, message: Message, now: Instant) -> Action {
        todo!();
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
