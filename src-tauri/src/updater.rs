use crate::models::{AddonInfo, DeleteAddonResult, UpdateRequest, UpdateResult};
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
    validate_addon_target(&request.addon, &addons_root)?;
    validate_download_url(&request.download_url)?;

    let client = Client::builder()
        .user_agent(format!("WowBox/{}", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("无法初始化下载器：{error}"))?;
    // CurseForge's REST API key is only used for API queries. The returned
    // CDN archive URL is public, so never forward either the built-in key or
    // a user key to the download host.
    let response = client
        .get(&request.download_url)
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

pub fn delete_addon(addon: AddonInfo, addons_root: PathBuf) -> Result<DeleteAddonResult, String> {
    if !addons_root.is_dir() {
        return Err(format!("插件根目录不存在：{}", addons_root.display()));
    }
    let addons_root =
        fs::canonicalize(addons_root).map_err(|error| format!("无法确认插件根目录：{error}"))?;
    validate_addon_target(&addon, &addons_root)?;

    let trash_parent = addons_root.join(".wowbox-trash");
    if let Ok(metadata) = fs::symlink_metadata(&trash_parent) {
        if metadata.file_type().is_symlink() {
            return Err("插件回收目录不能是符号链接。".into());
        }
    }
    fs::create_dir_all(&trash_parent).map_err(|error| format!("无法创建插件回收目录：{error}"))?;
    let canonical_trash_parent = fs::canonicalize(&trash_parent)
        .map_err(|error| format!("无法确认插件回收目录：{error}"))?;
    if canonical_trash_parent != trash_parent {
        return Err("插件回收目录不在已授权的插件目录中。".into());
    }

    let trash_root = trash_parent.join(format!(
        "{}-{}",
        Utc::now().format("%Y%m%d-%H%M%S-%3f"),
        safe_name(&addon.folder_name)
    ));
    fs::create_dir_all(&trash_root).map_err(|error| format!("无法创建插件回收目录：{error}"))?;

    let mut folders: BTreeSet<String> = addon.folders.iter().cloned().collect();
    folders.insert(addon.folder_name.clone());
    let mut removed_folders = Vec::new();

    for folder in folders {
        if !is_safe_folder_name(&folder) {
            rollback(&addons_root, &trash_root, &removed_folders);
            return Err(format!("插件目录名不安全：{folder}"));
        }
        let source = addons_root.join(&folder);
        if !source.exists() {
            continue;
        }
        let destination = trash_root.join(&folder);
        if let Err(error) = fs::rename(&source, &destination) {
            rollback(&addons_root, &trash_root, &removed_folders);
            return Err(format!("删除 {folder} 失败，已恢复插件：{error}"));
        }
        removed_folders.push(folder);
    }

    if removed_folders.is_empty() {
        return Err("没有找到可删除的插件目录。".into());
    }

    Ok(DeleteAddonResult {
        addon_id: addon.id,
        trash_path: trash_root.to_string_lossy().into_owned(),
        removed_folders,
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

fn validate_addon_target(addon: &AddonInfo, addons_root: &Path) -> Result<(), String> {
    let mut folders = addon.folders.clone();
    if !folders.iter().any(|folder| folder == &addon.folder_name) {
        folders.push(addon.folder_name.clone());
    }
    for folder in &folders {
        if !is_safe_folder_name(folder) {
            return Err(format!("插件目录名不安全：{folder}"));
        }
    }

    let expected_path = std::fs::canonicalize(addons_root.join(&addon.folder_name))
        .map_err(|error| format!("无法确认插件路径：{error}"))?;
    let provided_path =
        std::fs::canonicalize(&addon.path).map_err(|error| format!("无法确认插件路径：{error}"))?;
    if expected_path != provided_path {
        return Err("插件路径与已扫描目录不一致，已取消更新。".into());
    }
    Ok(())
}

fn validate_download_url(download_url: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(download_url)
        .map_err(|_| "插件下载地址无效，已取消更新。".to_string())?;
    let host = url
        .host_str()
        .map(|host| host.trim_end_matches('.').to_ascii_lowercase())
        .ok_or_else(|| "插件下载地址缺少主机名，已取消更新。".to_string())?;
    if url.scheme() != "https" || !(host == "forgecdn.net" || host.ends_with(".forgecdn.net")) {
        return Err("插件下载地址不是受信任的 CurseForge CDN，已取消更新。".into());
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

#[cfg(test)]
mod tests {
    use super::delete_addon;
    use crate::models::AddonInfo;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    fn test_addon(path: &std::path::Path) -> AddonInfo {
        AddonInfo {
            id: "folder:ExampleAddon".into(),
            title: "Example Addon".into(),
            notes: String::new(),
            author: String::new(),
            version: "1.0".into(),
            interface_version: "110200".into(),
            source: "unknown".into(),
            source_id: None,
            folder_name: "ExampleAddon".into(),
            folders: vec!["ExampleAddon".into(), "ExampleAddon_Config".into()],
            path: path.to_string_lossy().into_owned(),
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
    fn delete_moves_all_addon_folders_to_recoverable_trash() {
        let root = tempdir().expect("temporary add-on root");
        let primary = root.path().join("ExampleAddon");
        let companion = root.path().join("ExampleAddon_Config");
        fs::create_dir_all(&primary).expect("primary folder");
        fs::create_dir_all(&companion).expect("companion folder");
        fs::write(primary.join("ExampleAddon.toc"), "## Version: 1.0").expect("toc fixture");

        let addon = test_addon(&primary);

        let result = delete_addon(addon, root.path().to_path_buf()).expect("move to trash");

        assert!(!primary.exists());
        assert!(!companion.exists());
        assert!(std::path::Path::new(&result.trash_path)
            .join("ExampleAddon")
            .is_dir());
        assert!(std::path::Path::new(&result.trash_path)
            .join("ExampleAddon_Config")
            .is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn delete_rejects_symlinked_trash_directory() {
        let root = tempdir().expect("temporary add-on root");
        let external = tempdir().expect("external trash target");
        let primary = root.path().join("ExampleAddon");
        fs::create_dir_all(&primary).expect("primary folder");
        fs::write(primary.join("ExampleAddon.toc"), "## Version: 1.0").expect("toc fixture");
        symlink(external.path(), root.path().join(".wowbox-trash")).expect("trash symlink");

        let error = delete_addon(test_addon(&primary), root.path().to_path_buf())
            .expect_err("symlinked trash must be rejected");

        assert!(error.contains("符号链接"));
        assert!(primary.is_dir());
        assert!(external
            .path()
            .read_dir()
            .expect("external directory")
            .next()
            .is_none());
    }
}
