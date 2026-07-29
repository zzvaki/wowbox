use crate::{
    models::{UpdateRequest, UpdateResult},
    provider_config::curseforge_api_key,
};
use chrono::Utc;
use reqwest::Client;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Cursor, Read},
    path::{Component, Path, PathBuf},
};
use tempfile::tempdir_in;
use zip::ZipArchive;

const MAX_ARCHIVE_BYTES: u64 = 200 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 2_000;
const MAX_EXTRACTED_BYTES: u64 = 500 * 1024 * 1024;

pub async fn update_addon(
    request: UpdateRequest,
    addons_root: PathBuf,
) -> Result<UpdateResult, String> {
    if !addons_root.is_dir() {
        return Err(format!("插件根目录不存在：{}", addons_root.display()));
    }
    validate_update_target(&request, &addons_root)?;

    let client = Client::builder()
        .user_agent(format!("WowBox/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("无法初始化下载器：{error}"))?;
    let mut download = client.get(&request.download_url);
    if request.addon.source == "curseforge" {
        let api_key = curseforge_api_key(request.api_key.as_deref())
            .ok_or_else(|| "下载 CurseForge 文件需要 API Key。".to_string())?;
        download = download.header("x-api-key", api_key);
    }
    let response = download
        .send()
        .await
        .map_err(|error| format!("下载插件失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("更新源拒绝了下载请求：{error}"))?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_ARCHIVE_BYTES)
    {
        return Err("插件压缩包超过 200 MB 安全限制。".into());
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("读取插件压缩包失败：{error}"))?;
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err("插件压缩包超过 200 MB 安全限制。".into());
    }

    let staging = tempdir_in(&addons_root).map_err(|error| format!("无法创建临时目录：{error}"))?;
    let staged_root = staging.path().join("payload");
    fs::create_dir(&staged_root).map_err(|error| format!("无法创建解压目录：{error}"))?;
    let installed_folders = extract_archive(&bytes, &staged_root)?;
    if installed_folders.is_empty() {
        return Err("压缩包中没有找到插件目录。".into());
    }
    let known_folders: BTreeSet<&str> = request
        .addon
        .folders
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(request.addon.folder_name.as_str()))
        .collect();
    let unexpected_folders: Vec<&String> = installed_folders
        .iter()
        .filter(|folder| !known_folders.contains(folder.as_str()))
        .collect();
    if !unexpected_folders.is_empty() {
        return Err(format!(
            "压缩包包含未关联的顶层目录（{}），为避免覆盖其他插件已取消更新。",
            unexpected_folders
                .iter()
                .map(|folder| folder.as_str())
                .collect::<Vec<_>>()
                .join("、")
        ));
    }

    let backup_root = addons_root.join(".wowbox-backups").join(format!(
        "{}-{}",
        Utc::now().format("%Y%m%d-%H%M%S"),
        safe_name(&request.addon.folder_name)
    ));
    fs::create_dir_all(&backup_root).map_err(|error| format!("无法创建备份目录：{error}"))?;

    let mut folders_to_backup: BTreeSet<String> = request.addon.folders.iter().cloned().collect();
    folders_to_backup.insert(request.addon.folder_name.clone());
    let mut backed_up = Vec::new();

    for folder in &folders_to_backup {
        let source = addons_root.join(folder);
        if !source.exists() {
            continue;
        }
        let destination = backup_root.join(folder);
        if let Err(error) = fs::rename(&source, &destination) {
            rollback(&addons_root, &backup_root, &backed_up);
            return Err(format!("备份 {folder} 失败：{error}"));
        }
        backed_up.push(folder.clone());
    }

    let mut installed = Vec::new();
    for folder in &installed_folders {
        let source = staged_root.join(folder);
        let destination = addons_root.join(folder);
        if let Err(error) = fs::rename(&source, &destination) {
            for installed_folder in &installed {
                let path = addons_root.join(installed_folder);
                let _ = if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
            }
            rollback(&addons_root, &backup_root, &backed_up);
            return Err(format!("安装 {folder} 失败，已恢复旧版本：{error}"));
        }
        installed.push(folder.clone());
    }

    Ok(UpdateResult {
        addon_id: request.addon.id,
        version: request
            .addon
            .latest_version
            .unwrap_or(request.addon.version),
        backup_path: backup_root.to_string_lossy().into_owned(),
        installed_folders: installed,
    })
}

fn extract_archive(bytes: &[u8], destination: &Path) -> Result<Vec<String>, String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|error| format!("无效的 ZIP 文件：{error}"))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "插件压缩包包含超过 {MAX_ARCHIVE_ENTRIES} 个文件，已拒绝解压。"
        ));
    }
    let mut top_level = BTreeSet::new();
    let mut extracted_bytes = 0_u64;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取 ZIP 条目失败：{error}"))?;
        let Some(relative) = safe_zip_path(entry.name()) else {
            return Err(format!("ZIP 中包含不安全路径：{}", entry.name()));
        };
        if relative
            .components()
            .next()
            .and_then(component_name)
            .is_some_and(|name| name == "__MACOSX" || name.starts_with('.'))
        {
            continue;
        }
        let Some(first) = relative.components().next().and_then(component_name) else {
            continue;
        };
        top_level.insert(first.to_string());
        let output_path = destination.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| format!("创建目录失败：{error}"))?;
        } else {
            extracted_bytes = extracted_bytes
                .checked_add(entry.size())
                .ok_or_else(|| "插件解压大小超出安全限制。".to_string())?;
            if extracted_bytes > MAX_EXTRACTED_BYTES {
                return Err("插件解压后超过 500 MB 安全限制。".into());
            }
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).map_err(|error| format!("创建目录失败：{error}"))?;
            }
            let mut output =
                fs::File::create(&output_path).map_err(|error| format!("创建文件失败：{error}"))?;
            let remaining = MAX_EXTRACTED_BYTES - (extracted_bytes - entry.size());
            let copied = io::copy(&mut entry.by_ref().take(remaining + 1), &mut output)
                .map_err(|error| format!("解压文件失败：{error}"))?;
            if copied > remaining {
                return Err("插件解压后超过 500 MB 安全限制。".into());
            }
        }
    }

    let valid_folders = top_level
        .into_iter()
        .filter(|folder| destination.join(folder).is_dir())
        .collect();
    Ok(valid_folders)
}

fn validate_update_target(request: &UpdateRequest, addons_root: &Path) -> Result<(), String> {
    let mut folders = request.addon.folders.clone();
    if !folders
        .iter()
        .any(|folder| folder == &request.addon.folder_name)
    {
        folders.push(request.addon.folder_name.clone());
    }
    for folder in &folders {
        if !is_safe_folder_name(folder) {
            return Err(format!("插件目录名不安全：{folder}"));
        }
    }

    let expected_path = std::fs::canonicalize(addons_root.join(&request.addon.folder_name))
        .map_err(|error| format!("无法确认插件路径：{error}"))?;
    let provided_path = std::fs::canonicalize(&request.addon.path)
        .map_err(|error| format!("无法确认插件路径：{error}"))?;
    if expected_path != provided_path {
        return Err("插件路径与已扫描目录不一致，已取消更新。".into());
    }
    Ok(())
}

fn is_safe_folder_name(folder: &str) -> bool {
    !folder.is_empty()
        && Path::new(folder).components().count() == 1
        && matches!(
            Path::new(folder).components().next(),
            Some(Component::Normal(_))
        )
}

fn safe_zip_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return None;
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => safe.push(value),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => return None,
        }
    }
    Some(safe)
}

fn component_name(component: Component<'_>) -> Option<&str> {
    match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    }
}

fn rollback(addons_root: &Path, backup_root: &Path, folders: &[String]) {
    for folder in folders {
        let backup = backup_root.join(folder);
        let original = addons_root.join(folder);
        if backup.exists() && !original.exists() {
            let _ = fs::rename(backup, original);
        }
    }
}

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
