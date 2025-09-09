use anywho::anywho;
use iced::Theme;
use serde::{Deserialize, Serialize};

const CONFIG_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub theme: ApplicationTheme,
}

impl Config {
    pub async fn load(app_id: &str) -> Result<Self, anywho::Error> {
        use dirs;
        use std::fs;
        use tokio::task;

        let app_id = app_id.to_string();

        task::spawn_blocking(move || {
            let config_dir = dirs::config_dir()
                .ok_or_else(|| anywho!("Could not determine config directory"))?
                .join(&app_id);

            // create config directory if it doesn't exist
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)
                    .map_err(|e| anywho!("Failed to create config directory: {}", e))?;
            }

            let config_path = config_dir.join(format!("config_v{}.ron", CONFIG_VERSION));

            if config_path.exists() {
                let config_content = fs::read_to_string(&config_path)
                    .map_err(|e| anywho!("Failed to read config file: {}", e))?;

                ron::from_str(&config_content)
                    .map_err(|e| anywho!("Failed to parse config file: {}", e))
            } else {
                let config = Config::default();

                // Save default config
                let config_content =
                    ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())
                        .map_err(|e| anywho!("Failed to serialize config: {}", e))?;

                fs::write(&config_path, config_content)
                    .map_err(|e| anywho!("Failed to write config file: {}", e))?;

                Ok(config)
            }
        })
        .await
        .map_err(|e| anywho!("Task join error: {}", e))?
    }

    pub async fn save(self, app_id: &str) -> Result<(), anywho::Error> {
        use dirs;
        use std::fs;
        use tokio::task;

        let config_clone = self.clone();
        let app_id = app_id.to_string();

        task::spawn_blocking(move || {
            let config_dir = dirs::config_dir()
                .ok_or_else(|| anywho!("Could not determine config directory"))?
                .join(&app_id);

            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)
                    .map_err(|e| anywho!("Failed to create config directory: {}", e))?;
            }

            let config_path = config_dir.join(format!("config_v{}.ron", CONFIG_VERSION));

            let config_content =
                ron::ser::to_string_pretty(&config_clone, ron::ser::PrettyConfig::default())
                    .map_err(|e| anywho!("Failed to serialize config: {}", e))?;

            fs::write(&config_path, config_content)
                .map_err(|e| anywho!("Failed to write config file: {}", e))?;

            Ok(())
        })
        .await
        .map_err(|e| anywho!("Task join error: {}", e))?
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ApplicationTheme {
    #[default]
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl From<ApplicationTheme> for Theme {
    fn from(config_theme: ApplicationTheme) -> Self {
        match config_theme {
            ApplicationTheme::Light => Theme::Light,
            ApplicationTheme::Dark => Theme::Dark,
            ApplicationTheme::Dracula => Theme::Dracula,
            ApplicationTheme::Nord => Theme::Nord,
            ApplicationTheme::SolarizedLight => Theme::SolarizedLight,
            ApplicationTheme::SolarizedDark => Theme::SolarizedDark,
            ApplicationTheme::GruvboxLight => Theme::GruvboxLight,
            ApplicationTheme::GruvboxDark => Theme::GruvboxDark,
            ApplicationTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            ApplicationTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            ApplicationTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            ApplicationTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            ApplicationTheme::TokyoNight => Theme::TokyoNight,
            ApplicationTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            ApplicationTheme::TokyoNightLight => Theme::TokyoNightLight,
            ApplicationTheme::KanagawaWave => Theme::KanagawaWave,
            ApplicationTheme::KanagawaDragon => Theme::KanagawaDragon,
            ApplicationTheme::KanagawaLotus => Theme::KanagawaLotus,
            ApplicationTheme::Moonfly => Theme::Moonfly,
            ApplicationTheme::Nightfly => Theme::Nightfly,
            ApplicationTheme::Oxocarbon => Theme::Oxocarbon,
            ApplicationTheme::Ferra => Theme::Ferra,
        }
    }
}

/// Will fail for custom themes
impl TryFrom<&Theme> for ApplicationTheme {
    type Error = &'static str;

    fn try_from(theme: &Theme) -> Result<Self, Self::Error> {
        match theme {
            Theme::Light => Ok(ApplicationTheme::Light),
            Theme::Dark => Ok(ApplicationTheme::Dark),
            Theme::Dracula => Ok(ApplicationTheme::Dracula),
            Theme::Nord => Ok(ApplicationTheme::Nord),
            Theme::SolarizedLight => Ok(ApplicationTheme::SolarizedLight),
            Theme::SolarizedDark => Ok(ApplicationTheme::SolarizedDark),
            Theme::GruvboxLight => Ok(ApplicationTheme::GruvboxLight),
            Theme::GruvboxDark => Ok(ApplicationTheme::GruvboxDark),
            Theme::CatppuccinLatte => Ok(ApplicationTheme::CatppuccinLatte),
            Theme::CatppuccinFrappe => Ok(ApplicationTheme::CatppuccinFrappe),
            Theme::CatppuccinMacchiato => Ok(ApplicationTheme::CatppuccinMacchiato),
            Theme::CatppuccinMocha => Ok(ApplicationTheme::CatppuccinMocha),
            Theme::TokyoNight => Ok(ApplicationTheme::TokyoNight),
            Theme::TokyoNightStorm => Ok(ApplicationTheme::TokyoNightStorm),
            Theme::TokyoNightLight => Ok(ApplicationTheme::TokyoNightLight),
            Theme::KanagawaWave => Ok(ApplicationTheme::KanagawaWave),
            Theme::KanagawaDragon => Ok(ApplicationTheme::KanagawaDragon),
            Theme::KanagawaLotus => Ok(ApplicationTheme::KanagawaLotus),
            Theme::Moonfly => Ok(ApplicationTheme::Moonfly),
            Theme::Nightfly => Ok(ApplicationTheme::Nightfly),
            Theme::Oxocarbon => Ok(ApplicationTheme::Oxocarbon),
            Theme::Ferra => Ok(ApplicationTheme::Ferra),
            Theme::Custom(_) => Err("Custom themes cannot be converted to ConfigTheme"),
        }
    }
}
