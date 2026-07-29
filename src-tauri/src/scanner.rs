use crate::models::{AddonInfo, GameInstallation};
use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

const PRODUCT_FOLDERS: [(&str, &str, &str); 9] = [
    ("_retail_", "retail", "正式服"),
    ("_classic_", "classic", "经典进度服"),
    ("_classic_era_", "classic_era", "经典旧世"),
    ("_anniversary_", "classic_anniversary", "周年纪念服"),
    ("_classic_titan_", "classic_titan", "泰坦重铸时光服"),
    // Retained for older installations created before the current folder layout.
    ("_classic_anniversary_", "classic_anniversary", "周年纪念服"),
    ("_ptr_", "ptr", "测试服"),
    ("_classic_ptr_", "classic_ptr", "经典测试服"),
    ("_beta_", "beta", "Beta"),
];

pub fn detect_installations(custom_root: Option<String>) -> Result<Vec<GameInstallation>, String> {
    let roots = candidate_roots(custom_root);
    let root = roots
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| "没有找到 World of Warcraft 目录，请在设置中手动选择。".to_string())?;

    if is_addons_directory(&root) {
        return Ok(vec![installation_from_addons_path(&root)]);
    }

    let normalized_root = normalize_wow_root(root);
    let mut installations = Vec::new();

    for (product_folder, flavor, label) in PRODUCT_FOLDERS {
        let product_path = normalized_root.join(product_folder);
        let addons_path = product_path.join("Interface").join("AddOns");
        if !addons_path.is_dir() {
            continue;
        }
        installations.push(GameInstallation {
            id: flavor_id(flavor, product_folder),
            flavor: flavor.to_string(),
            label: label.to_string(),
            product_folder: product_folder.to_string(),
            path: path_string(&product_path),
            addons_path: path_string(&addons_path),
            addon_count: count_addon_folders(&addons_path),
            available: true,
        });
    }

    if installations.is_empty() {
        Err(format!(
            "目录 {} 中没有找到可用的客户端版本。",
            normalized_root.display()
        ))
    } else {
        Ok(installations)
    }
}

pub fn scan_addons(addons_path: &str, _flavor: &str) -> Result<Vec<AddonInfo>, String> {
    let root = Path::new(addons_path);
    if !root.is_dir() {
        return Err(format!("插件目录不存在：{}", root.display()));
    }

    let mut grouped: BTreeMap<String, AddonInfo> = BTreeMap::new();
    let entries = fs::read_dir(root).map_err(|error| format!("无法读取插件目录：{error}"))?;

    for entry in entries.flatten() {
        let folder_path = entry.path();
        if !folder_path.is_dir() || should_ignore_folder(&folder_path) {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().into_owned();
        let Some(toc_path) = find_toc_file(&folder_path, &folder_name) else {
            continue;
        };
        let metadata = parse_toc(&toc_path)?;
        let (source, source_id) = detect_source(&metadata);
        let group_id = match &source_id {
            Some(id) => format!("{source}:{id}"),
            None => format!("local:{}", folder_name.to_lowercase()),
        };
        let modified_at = folder_modified_at(&folder_path);

        if let Some(existing) = grouped.get_mut(&group_id) {
            existing.folders.push(folder_name.clone());
            let title = clean_wow_text(&localized_value(&metadata, "title"));
            let notes = clean_wow_text(&localized_value(&metadata, "notes"));
            let author = clean_wow_text(metadata.get("author").map(String::as_str).unwrap_or(""));
            let version = metadata
                .get("version")
                .filter(|value| !value.trim().is_empty());
            let interface_version = metadata.get("interface");

            // A package's auxiliary folder can be returned before its main folder.
            // Prefer the first TOC with a real title over a synthetic folder-name title.
            if !title.is_empty() && existing.title == existing.folder_name {
                existing.title = title;
                existing.folder_name = folder_name;
                existing.path = path_string(&folder_path);
            }
            if existing.notes.is_empty() && !notes.is_empty() {
                existing.notes = notes;
            }
            if existing.author.is_empty() && !author.is_empty() {
                existing.author = author;
            }
            if existing.version == "\u{672a}\u{77e5}" {
                if let Some(version) = version {
                    existing.version = version.clone();
                }
            }
            if existing.interface_version == "\u{672a}\u{77e5}" {
                if let Some(interface_version) = interface_version {
                    existing.interface_version = interface_version.clone();
                }
            }
            if existing.modified_at.is_none() {
                existing.modified_at = modified_at;
            }
            continue;
        }

        let title = localized_value(&metadata, "title");
        let addon = AddonInfo {
            id: group_id.clone(),
            title: if title.is_empty() {
                folder_name.clone()
            } else {
                clean_wow_text(&title)
            },
            notes: clean_wow_text(&localized_value(&metadata, "notes")),
            author: clean_wow_text(metadata.get("author").map(String::as_str).unwrap_or("")),
            version: metadata
                .get("version")
                .cloned()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "未知".into()),
            interface_version: metadata
                .get("interface")
                .cloned()
                .unwrap_or_else(|| "未知".into()),
            source: source.to_string(),
            source_id,
            folder_name: folder_name.clone(),
            folders: vec![folder_name],
            path: path_string(&folder_path),
            status: if source == "unknown" {
                "untracked".into()
            } else {
                "current".into()
            },
            latest_version: None,
            latest_file_id: None,
            latest_download_url: None,
            website_url: None,
            error: None,
            modified_at,
        };
        grouped.insert(group_id, addon);
    }

    let mut addons: Vec<_> = grouped.into_values().collect();
    addons.sort_by_key(|addon| addon.title.to_lowercase());
    Ok(addons)
}

fn candidate_roots(custom_root: Option<String>) -> Vec<PathBuf> {
    if let Some(root) = custom_root.filter(|value| !value.trim().is_empty()) {
        return vec![PathBuf::from(root)];
    }

    let mut candidates = Vec::new();
    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from("/Applications/World of Warcraft"));
        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join("Applications").join("World of Warcraft"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from(r"C:\Program Files (x86)\World of Warcraft"));
        candidates.push(PathBuf::from(r"C:\Program Files\World of Warcraft"));
        if let Ok(program_files) = std::env::var("PROGRAMFILES(X86)") {
            candidates.push(PathBuf::from(program_files).join("World of Warcraft"));
        }
        if let Ok(program_files) = std::env::var("PROGRAMFILES") {
            candidates.push(PathBuf::from(program_files).join("World of Warcraft"));
        }
    }
    candidates
}

fn normalize_wow_root(path: PathBuf) -> PathBuf {
    let is_product_folder = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| PRODUCT_FOLDERS.iter().any(|item| item.0 == name));
    if is_product_folder {
        path.parent().map(Path::to_path_buf).unwrap_or(path)
    } else {
        path
    }
}

fn is_addons_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("AddOns"))
}

fn installation_from_addons_path(addons_path: &Path) -> GameInstallation {
    let product_path = addons_path
        .parent()
        .and_then(Path::parent)
        .unwrap_or(addons_path);
    let product_folder = product_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("custom");
    let (flavor, label) = PRODUCT_FOLDERS
        .iter()
        .find(|item| item.0 == product_folder)
        .map(|item| (item.1, item.2))
        .unwrap_or(("retail", "自定义客户端"));
    GameInstallation {
        id: flavor_id(flavor, product_folder),
        flavor: flavor.into(),
        label: label.into(),
        product_folder: product_folder.into(),
        path: path_string(product_path),
        addons_path: path_string(addons_path),
        addon_count: count_addon_folders(addons_path),
        available: true,
    }
}

fn flavor_id(flavor: &str, product_folder: &str) -> String {
    format!("{flavor}-{}", product_folder.trim_matches('_'))
}

fn count_addon_folders(root: &Path) -> usize {
    fs::read_dir(root)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry.path().is_dir()
                        && !should_ignore_folder(&entry.path())
                        && find_toc_file(&entry.path(), &entry.file_name().to_string_lossy())
                            .is_some()
                })
                .count()
        })
        .unwrap_or(0)
}

fn should_ignore_folder(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_none_or(|name| name.starts_with('.') || name == "__MACOSX")
}

fn find_toc_file(folder: &Path, folder_name: &str) -> Option<PathBuf> {
    let exact = folder.join(format!("{folder_name}.toc"));
    if exact.is_file() {
        return Some(exact);
    }
    fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("toc"))
        })
}

fn parse_toc(path: &Path) -> Result<HashMap<String, String>, String> {
    let bytes = fs::read(path).map_err(|error| format!("无法读取 {}：{error}", path.display()))?;
    let content = String::from_utf8_lossy(&bytes);
    let mut metadata = HashMap::new();
    for line in content.lines() {
        let line = line.trim_start_matches('\u{feff}').trim();
        let Some(rest) = line.strip_prefix("##") else {
            continue;
        };
        let Some((key, value)) = rest.split_once(':') else {
            continue;
        };
        metadata.insert(key.trim().to_lowercase(), value.trim().to_string());
    }
    Ok(metadata)
}

fn localized_value(metadata: &HashMap<String, String>, key: &str) -> String {
    metadata
        .get(&format!("{key}-zhcn"))
        .or_else(|| metadata.get(key))
        .cloned()
        .unwrap_or_default()
}

fn detect_source(metadata: &HashMap<String, String>) -> (&'static str, Option<String>) {
    for key in ["x-curse-project-id", "x-curse-projectid"] {
        if let Some(value) = metadata.get(key).filter(|value| !value.trim().is_empty()) {
            return ("curseforge", Some(value.trim().to_string()));
        }
    }
    for key in ["x-wowi-id", "x-wowinterface-id"] {
        if let Some(value) = metadata.get(key).filter(|value| !value.trim().is_empty()) {
            return ("wowinterface", Some(value.trim().to_string()));
        }
    }
    ("unknown", None)
}

fn clean_wow_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '|' && index + 1 < chars.len() {
            match chars[index + 1] {
                'c' | 'C' if index + 9 < chars.len() => {
                    index += 10;
                    continue;
                }
                'r' | 'R' => {
                    index += 2;
                    continue;
                }
                _ => {}
            }
        }
        output.push(chars[index]);
        index += 1;
    }
    output.trim().to_string()
}

fn folder_modified_at(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let datetime: DateTime<Utc> = DateTime::<Utc>::from(modified);
    Some(datetime.to_rfc3339())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::{detect_installations, scan_addons};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_current_china_client_folder_layout() {
        let temporary_directory = tempdir().expect("create temporary directory");
        for product_folder in [
            "_anniversary_",
            "_classic_",
            "_classic_era_",
            "_classic_titan_",
        ] {
            fs::create_dir_all(
                temporary_directory
                    .path()
                    .join(product_folder)
                    .join("Interface")
                    .join("AddOns"),
            )
            .expect("create client AddOns directory");
        }

        let installations = detect_installations(Some(
            temporary_directory.path().to_string_lossy().into_owned(),
        ))
        .expect("detect current client directories");

        assert_eq!(installations.len(), 4);
        assert!(installations.iter().any(|item| {
            item.product_folder == "_anniversary_" && item.flavor == "classic_anniversary"
        }));
        assert!(installations.iter().any(|item| {
            item.product_folder == "_classic_titan_" && item.flavor == "classic_titan"
        }));
    }

    #[test]
    fn scans_localized_toc_metadata_and_merges_companion_folders() {
        let temporary_directory = tempdir().expect("create temporary directory");
        let addons_path = temporary_directory.path().join("Interface").join("AddOns");
        let details_path = addons_path.join("Details");
        let storage_path = addons_path.join("Details_DataStorage");
        let local_path = addons_path.join("LocalOnly");
        fs::create_dir_all(&details_path).expect("create Details directory");
        fs::create_dir_all(&storage_path).expect("create Details storage directory");
        fs::create_dir_all(&local_path).expect("create local directory");

        fs::write(
            details_path.join("Details.toc"),
            "## Interface: 110200\n## Title: |cff00ff00Details!|r\n## Title-zhCN: Details! \u{4f24}\u{5bb3}\u{7edf}\u{8ba1}\n## Notes-zhCN: \u{56e2}\u{961f}\u{6218}\u{6597}\u{6570}\u{636e}\n## Author: Terciob\n## Version: 11.2.0\n## X-Curse-Project-ID: 3358\n",
        )
        .expect("write Details toc");
        fs::write(
            storage_path.join("Details_DataStorage.toc"),
            "## Interface: 110200\n## X-Curse-Project-ID: 3358\n",
        )
        .expect("write Details companion toc");
        fs::write(
            local_path.join("LocalOnly.toc"),
            "## Title-zhCN: \u{672c}\u{5730}\u{63d2}\u{4ef6}\n## Version: 1.0.0\n",
        )
        .expect("write local toc");

        let addons = scan_addons(&addons_path.to_string_lossy(), "retail").expect("scan addons");
        let details = addons
            .iter()
            .find(|addon| addon.id == "curseforge:3358")
            .expect("merged CurseForge addon");
        let local = addons
            .iter()
            .find(|addon| addon.id == "local:localonly")
            .expect("untracked local addon");

        assert_eq!(addons.len(), 2);
        assert_eq!(details.title, "Details! \u{4f24}\u{5bb3}\u{7edf}\u{8ba1}");
        assert_eq!(
            details.notes,
            "\u{56e2}\u{961f}\u{6218}\u{6597}\u{6570}\u{636e}"
        );
        assert_eq!(details.author, "Terciob");
        assert_eq!(details.version, "11.2.0");
        assert_eq!(details.interface_version, "110200");
        assert_eq!(details.source, "curseforge");
        assert_eq!(details.source_id.as_deref(), Some("3358"));
        assert!(details.folders.iter().any(|folder| folder == "Details"));
        assert!(details
            .folders
            .iter()
            .any(|folder| folder == "Details_DataStorage"));
        assert_eq!(local.status, "untracked");
        assert_eq!(local.title, "\u{672c}\u{5730}\u{63d2}\u{4ef6}");
    }
}
