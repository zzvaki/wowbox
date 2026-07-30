use crate::models::{AddonInfo, UpdateCheckResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Component, Path, PathBuf},
};
use tauri::Manager;

const ADDON_PACKAGES_FILE: &str = "addon-packages.json";

#[derive(Default, Deserialize, Serialize)]
struct AddonPackageRegistry {
    roots: BTreeMap<String, Vec<RegisteredPackage>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct RegisteredPackage {
    source: String,
    source_id: String,
    folders: Vec<String>,
}

pub fn known_sources(
    app: &tauri::AppHandle,
    addons_root: &Path,
) -> HashMap<String, (String, String)> {
    let registry = load_registry(app);
    let Some(packages) = registry.roots.get(&path_key(addons_root)) else {
        return HashMap::new();
    };
    folder_sources(packages)
}

pub fn remember_packages(
    app: &tauri::AppHandle,
    addons_root: &Path,
    addons: &[AddonInfo],
    results: &[UpdateCheckResult],
) -> Result<(), String> {
    let mut registry = load_registry(app);
    let packages = registry.roots.entry(path_key(addons_root)).or_default();
    update_packages(packages, addons, results);
    persist_registry(app, &registry)
}

fn update_packages(
    packages: &mut Vec<RegisteredPackage>,
    addons: &[AddonInfo],
    results: &[UpdateCheckResult],
) {
    let addons_by_id = addons
        .iter()
        .map(|addon| (addon.id.as_str(), addon))
        .collect::<HashMap<_, _>>();
    let mut updates = BTreeMap::<String, BTreeSet<String>>::new();

    for addon in addons {
        let Some(source_id) = addon
            .source_id
            .as_ref()
            .filter(|_| addon.source == "curseforge")
        else {
            continue;
        };
        updates.entry(source_id.clone()).or_default().extend(
            addon
                .folders
                .iter()
                .chain(std::iter::once(&addon.folder_name))
                .chain(addon.package_folders.iter())
                .filter(|folder| is_safe_folder_name(folder))
                .cloned(),
        );
    }

    for result in results {
        let Some(source_id) = result.source_id.as_ref() else {
            continue;
        };
        let folders = updates.entry(source_id.clone()).or_default();
        folders.extend(
            result
                .package_folders
                .iter()
                .filter(|folder| is_safe_folder_name(folder))
                .cloned(),
        );
        if let Some(addon) = addons_by_id.get(result.addon_id.as_str()) {
            folders.extend(
                addon
                    .folders
                    .iter()
                    .chain(std::iter::once(&addon.folder_name))
                    .filter(|folder| is_safe_folder_name(folder))
                    .cloned(),
            );
        }
    }

    *packages = updates
        .into_iter()
        .map(|(source_id, folders)| RegisteredPackage {
            source: "curseforge".into(),
            source_id,
            folders: folders.into_iter().collect(),
        })
        .collect();
}

fn folder_sources(packages: &[RegisteredPackage]) -> HashMap<String, (String, String)> {
    let mut candidates = HashMap::<String, BTreeSet<(String, String)>>::new();
    for package in packages {
        for folder in &package.folders {
            if !is_safe_folder_name(folder) {
                continue;
            }
            candidates
                .entry(folder.to_ascii_lowercase())
                .or_default()
                .insert((package.source.clone(), package.source_id.clone()));
        }
    }
    candidates
        .into_iter()
        .filter_map(|(folder, sources)| {
            (sources.len() == 1)
                .then(|| sources.into_iter().next().map(|source| (folder, source)))
                .flatten()
        })
        .collect()
}

fn load_registry(app: &tauri::AppHandle) -> AddonPackageRegistry {
    registry_path(app)
        .and_then(|path| fs::read(path).ok())
        .and_then(|contents| serde_json::from_slice(&contents).ok())
        .unwrap_or_default()
}

fn persist_registry(app: &tauri::AppHandle, registry: &AddonPackageRegistry) -> Result<(), String> {
    let path = registry_path(app).ok_or_else(|| "无法确定插件关联配置文件位置。".to_string())?;
    let directory = path
        .parent()
        .ok_or_else(|| "无法确定插件关联配置目录。".to_string())?;
    fs::create_dir_all(directory).map_err(|error| format!("无法创建应用配置目录：{error}"))?;
    let contents = serde_json::to_vec_pretty(registry)
        .map_err(|error| format!("无法保存插件目录关联：{error}"))?;
    fs::write(path, contents).map_err(|error| format!("无法写入插件目录关联：{error}"))
}

fn registry_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|directory| directory.join(ADDON_PACKAGES_FILE))
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn is_safe_folder_name(folder: &str) -> bool {
    !folder.is_empty()
        && Path::new(folder).components().count() == 1
        && matches!(
            Path::new(folder).components().next(),
            Some(Component::Normal(_))
        )
}

#[cfg(test)]
mod tests {
    use super::{folder_sources, update_packages, RegisteredPackage};
    use crate::models::{AddonInfo, UpdateCheckResult};

    fn addon(folder_name: &str) -> AddonInfo {
        AddonInfo {
            id: format!("local:{}", folder_name.to_ascii_lowercase()),
            title: folder_name.into(),
            notes: String::new(),
            author: String::new(),
            version: "1.0".into(),
            interface_version: "110200".into(),
            source: "unknown".into(),
            source_id: None,
            folder_name: folder_name.into(),
            folders: vec![folder_name.into()],
            package_folders: Vec::new(),
            path: format!("/AddOns/{folder_name}"),
            status: "untracked".into(),
            latest_version: None,
            latest_file_id: None,
            latest_download_url: None,
            website_url: None,
            error: None,
            modified_at: None,
        }
    }

    #[test]
    fn remembers_current_remote_modules_and_prunes_stale_packages() {
        let addons = vec![addon("Details")];
        let results = vec![UpdateCheckResult {
            addon_id: addons[0].id.clone(),
            status: "current".into(),
            title: None,
            author: None,
            summary: None,
            source_id: Some("3358".into()),
            latest_version: None,
            latest_file_id: None,
            download_url: None,
            website_url: None,
            package_folders: vec!["Details".into(), "Details_DataStorage".into()],
            error: None,
        }];
        let mut packages = vec![RegisteredPackage {
            source: "curseforge".into(),
            source_id: "999".into(),
            folders: vec!["SharedFolder".into(), "Details_DataStorage".into()],
        }];

        update_packages(&mut packages, &addons, &results);
        let sources = folder_sources(&packages);

        assert_eq!(
            sources.get("details"),
            Some(&("curseforge".into(), "3358".into()))
        );
        assert_eq!(
            sources.get("details_datastorage"),
            Some(&("curseforge".into(), "3358".into()))
        );
        assert!(!sources.contains_key("sharedfolder"));
        assert!(!packages.iter().any(|package| package.source_id == "999"));
    }

    #[test]
    fn ignores_a_folder_claimed_by_two_current_packages() {
        let packages = vec![
            RegisteredPackage {
                source: "curseforge".into(),
                source_id: "1".into(),
                folders: vec!["SharedFolder".into()],
            },
            RegisteredPackage {
                source: "curseforge".into(),
                source_id: "2".into(),
                folders: vec!["SharedFolder".into()],
            },
        ];

        let sources = folder_sources(&packages);
        assert!(!sources.contains_key("sharedfolder"));
    }
}
