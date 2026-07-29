mod models;
mod providers;
mod scanner;
mod updater;
mod version;

use models::{AddonInfo, GameInstallation, UpdateCheckResult, UpdateRequest, UpdateResult};
use std::{collections::HashSet, path::PathBuf, sync::Mutex};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct ManagedAddonRoots(Mutex<HashSet<PathBuf>>);

#[derive(Default)]
struct AuthorizedGameRoots(Mutex<HashSet<PathBuf>>);

#[tauri::command]
fn detect_installations(
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
    let installations = scanner::detect_installations(custom_root)?;
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
    authorized_game_roots
        .0
        .lock()
        .map_err(|_| "游戏目录状态不可用。".to_string())?
        .insert(root.clone());
    Ok(Some(root.to_string_lossy().into_owned()))
}

#[tauri::command]
fn scan_addons(
    addons_path: String,
    flavor: String,
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
    scanner::scan_addons(&addons_path, &flavor)
}

#[tauri::command]
async fn check_updates(
    addons: Vec<AddonInfo>,
    flavor: String,
    curseforge_api_key: Option<String>,
) -> Result<Vec<UpdateCheckResult>, String> {
    providers::check_updates(addons, flavor, curseforge_api_key).await
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
        .manage(AuthorizedGameRoots::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect_installations,
            choose_game_root,
            scan_addons,
            check_updates,
            update_addon
        ])
        .run(tauri::generate_context!())
        .expect("error while running WowBox");
}
