use agrm_settings::Settings;

const AGRM_NAME: &str = "agrm";
const CONFIG_FILE: &str = "agrm.toml";
const DOT_CONFIG_FILE: &str = ".agrm.toml";

/// When using as the first time, agrm will help user to generate a config file.
/// As usual, agrm loads settings from these sources and merge them:
///     For Windows:
///         1. ${AppData}/Roaming/agrm/agrm.toml
///         2. ${AppData}/.agrm.toml
///         3. ${AppData}/agrm/.agrm.toml
///     For Unix:
///         1. ${HOME}/.config/agrm/agrm.toml
///         2. ${HOME}/.config/agrm.toml
///         3. ${HOME}/.agrm.toml
pub fn load() -> Settings {
    let sources = default_sources();
    let settings_files_exists = exists_config(&sources);
    let s = match settings_files_exists {
        true => Settings::from_multi_files(sources),
        false => Ok(init_once()),
    };

    let s = match s {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Loading settings error.");
            eprintln!("Please check your configuration file.");

            Settings::default()
        }
    };

    s
}

fn init_once() -> Settings {
    todo!()
}

fn exists_config(paths: &[String]) -> bool {
    paths
        .iter()
        .any(|path| std::path::Path::new(&path).exists())
}

#[cfg(target_family = "windows")]
fn default_sources() -> Vec<String> {
    let app_data = std::env::var("AppData").unwrap();
    vec![
        format!("{}/Roaming/{}", app_data, CONFIG_FILE),
        format!("{}/{}", app_data, DOT_CONFIG_FILE),
        format!("{}/{}/{}", app_data, AGRM_NAME, DOT_CONFIG_FILE),
    ]
}

#[cfg(target_family = "unix")]
fn default_sources() -> Vec<String> {
    let home = std::env::var("HOME").unwrap();
    vec![
        format!("{}/.config/{}/{}", home, AGRM_NAME, CONFIG_FILE),
        format!("{}/.config/{}", home, CONFIG_FILE),
        format!("{}/{}", home, DOT_CONFIG_FILE),
    ]
}
