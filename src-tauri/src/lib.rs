mod models;
mod provider_config;
mod providers;
mod scanner;
mod updater;
mod version;

use models::{AddonInfo, GameInstallation, UpdateCheckResult, UpdateRequest, UpdateResult};
use std::{collections::HashSet, fs, path::PathBuf, sync::Mutex};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

const AUTHORIZED_GAME_ROOTS_FILE: &str = "authorized-game-roots.json";

#[derive(Default)]
struct ManagedAddonRoots(Mutex<HashSet<PathBuf>>);

#[derive(Default)]
struct AuthorizedGameRoots(Mutex<HashSet<PathBuf>>);

fn load_authorized_game_roots(app: &tauri::AppHandle) -> HashSet<PathBuf> {
    let Ok(config_directory) = app.path().app_config_dir() else {
        return HashSet::new();
    };
    let roots_path = config_directory.join(AUTHORIZED_GAME_ROOTS_FILE);
    let Ok(contents) = fs::read(&roots_path) else {
        return HashSet::new();
    };
    let Ok(saved_roots) = serde_json::from_slice::<Vec<String>>(&contents) else {
        return HashSet::new();
    };
    saved_roots
        .into_iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect()
}

fn persist_authorized_game_roots(
    app: &tauri::AppHandle,
    roots: &HashSet<PathBuf>,
) -> Result<(), String> {
    let config_directory = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法确定应用配置目录：{error}"))?;
    fs::create_dir_all(&config_directory)
        .map_err(|error| format!("无法创建应用配置目录：{error}"))?;
    let mut saved_roots: Vec<String> = roots
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    saved_roots.sort_unstable();
    let contents = serde_json::to_vec(&saved_roots)
        .map_err(|error| format!("无法保存已授权游戏目录：{error}"))?;
    fs::write(config_directory.join(AUTHORIZED_GAME_ROOTS_FILE), contents)
        .map_err(|error| format!("无法写入已授权游戏目录：{error}"))
}

#[tauri::command]
fn detect_installations(
    app: tauri::AppHandle,
    custom_root: Option<String>,
    authorized_game_roots: State<'_, AuthorizedGameRoots>,
    managed_addon_roots: State<'_, ManagedAddonRoots>,
) -> Result<Vec<GameInstallation>, String> {
    let custom_root = match custom_root.filter(|root| !root.trim().is_empty()) {
        Some(root) => {
            let root = std::fs::canonicalize(root)
                .map_err(|error| format!("无法确认所选游戏目录：{error}"))?;
            let is_authorized = authorized_game_roots
                .0
                .lock()
                .map_err(|_| "游戏目录状态不可用。".to_string())?
                .contains(&root);
            if !is_authorized {
                return Err("请使用“选择”按钮通过系统目录选择框授权游戏目录。".into());
            }
            Some(root.to_string_lossy().into_owned())
        }
        None => None,
    };
    let is_custom_root = custom_root.is_some();
    let installations = scanner::detect_installations(custom_root)?;
    let persisted_roots = if is_custom_root {
        let mut roots = authorized_game_roots
            .0
            .lock()
            .map_err(|_| "游戏目录状态不可用。".to_string())?;
        for installation in &installations {
            let installation_root = std::fs::canonicalize(&installation.path)
                .map_err(|error| format!("无法确认客户端目录：{error}"))?;
            roots.insert(installation_root);
        }
        Some(roots.clone())
    } else {
        None
    };
    if let Some(roots) = persisted_roots {
        persist_authorized_game_roots(&app, &roots)?;
    }
    let mut roots = managed_addon_roots
        .0
        .lock()
        .map_err(|_| "插件目录状态不可用。".to_string())?;
    for installation in &installations {
        let addons_root = std::fs::canonicalize(&installation.addons_path)
            .map_err(|error| format!("无法确认插件目录：{error}"))?;
        roots.insert(addons_root);
    }
    Ok(installations)
}

#[tauri::command]
async fn choose_game_root(
    app: tauri::AppHandle,
    authorized_game_roots: State<'_, AuthorizedGameRoots>,
) -> Result<Option<String>, String> {
    let selected = app
        .dialog()
        .file()
        .set_title("选择 World of Warcraft 目录")
        .blocking_pick_folder();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let root = selected
        .into_path()
        .map_err(|error| format!("无法读取所选游戏目录：{error}"))?;
    let root =
        std::fs::canonicalize(root).map_err(|error| format!("无法确认所选游戏目录：{error}"))?;
    let persisted_roots = {
        let mut roots = authorized_game_roots
            .0
            .lock()
            .map_err(|_| "游戏目录状态不可用。".to_string())?;
        roots.insert(root.clone());
        roots.clone()
    };
    persist_authorized_game_roots(&app, &persisted_roots)?;
    Ok(Some(root.to_string_lossy().into_owned()))
}

#[tauri::command]
fn sync_authorized_game_roots(
    app: tauri::AppHandle,
    configured_paths: Vec<String>,
    authorized_game_roots: State<'_, AuthorizedGameRoots>,
) -> Result<(), String> {
    let mut roots = authorized_game_roots
        .0
        .lock()
        .map_err(|_| "游戏目录状态不可用。".to_string())?;
    let retained_roots: HashSet<PathBuf> = configured_paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .filter_map(|path| std::fs::canonicalize(path).ok())
        .filter(|path| roots.contains(path))
        .collect();
    *roots = retained_roots.clone();
    drop(roots);
    persist_authorized_game_roots(&app, &retained_roots)
}

#[tauri::command]
fn scan_addons(
    addons_path: String,
    flavor: String,
    locale: String,
    managed_roots: State<'_, ManagedAddonRoots>,
) -> Result<Vec<AddonInfo>, String> {
    let root = std::fs::canonicalize(&addons_path)
        .map_err(|error| format!("无法确认插件目录：{error}"))?;
    let is_managed = managed_roots
        .0
        .lock()
        .map_err(|_| "插件目录状态不可用。".to_string())?
        .contains(&root);
    if !is_managed {
        return Err("插件目录尚未由客户端检测流程授权，请先重新检测游戏目录。".into());
    }
    scanner::scan_addons(&addons_path, &flavor, &locale)
}

#[tauri::command]
async fn check_updates(
    addons: Vec<AddonInfo>,
    flavor: String,
    provider: String,
    user_curseforge_api_key: Option<String>,
) -> Result<Vec<UpdateCheckResult>, String> {
    providers::check_updates(addons, flavor, provider, user_curseforge_api_key).await
}

#[tauri::command]
async fn update_addon(
    request: UpdateRequest,
    managed_roots: State<'_, ManagedAddonRoots>,
) -> Result<UpdateResult, String> {
    let addons_root = std::path::Path::new(&request.addon.path)
        .parent()
        .ok_or_else(|| "无法确定插件根目录。".to_string())?;
    let addons_root = std::fs::canonicalize(addons_root)
        .map_err(|error| format!("无法确认插件根目录：{error}"))?;
    let is_managed = managed_roots
        .0
        .lock()
        .map_err(|_| "插件目录状态不可用。".to_string())?
        .contains(&addons_root);
    if !is_managed {
        return Err("插件目录尚未由本次会话扫描，请先重新扫描后再更新。".into());
    }
    updater::update_addon(request, addons_root).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ManagedAddonRoots::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let authorized_roots = load_authorized_game_roots(app.handle());
            app.manage(AuthorizedGameRoots(Mutex::new(authorized_roots)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            detect_installations,
            choose_game_root,
            sync_authorized_game_roots,
            scan_addons,
            check_updates,
            update_addon
        ])
        .run(tauri::generate_context!())
        .expect("error while running WowBox");
}
