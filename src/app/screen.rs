// SPDX-License-Identifier: GPL-3.0-only

pub mod about;
pub mod config;
pub mod rollback;
pub mod system_status;
pub mod update_applications;
pub mod update_system;

pub use about::About;
pub use config::Config;
pub use rollback::Rollback;
pub use system_status::SystemStatus;
pub use update_applications::UpdateApplications;
pub use update_system::UpdateSystem;

pub enum Screen {
    Welcome,
    UpdateSystem(UpdateSystem),
    UpdateApplications(UpdateApplications),
    Rollback(Rollback),
    SystemStatus(SystemStatus),
    Config(Config),
    About(About),
}
