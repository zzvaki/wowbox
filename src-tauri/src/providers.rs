use crate::{
    models::{AddonInfo, UpdateCheckResult},
    version::is_remote_newer,
};
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const WOW_GAME_ID: u64 = 1;

pub async fn check_updates(
    addons: Vec<AddonInfo>,
    flavor: String,
    curseforge_api_key: Option<String>,
) -> Result<Vec<UpdateCheckResult>, String> {
    let client = Client::builder()
        .user_agent(format!("WowBox/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("无法初始化网络客户端：{error}"))?;

    // Never fall back to CurseForge's unfiltered files endpoint: doing so could
    // install a Retail package into a Classic client when version-type lookup
    // fails or the selected game flavor is unknown.
    let version_type = match curseforge_api_key.as_deref() {
        Some(key) if !key.trim().is_empty() => {
            find_curseforge_version_type(&client, key, &flavor).await
        }
        _ => Ok(None),
    };

    let results = stream::iter(addons.into_iter().map(|addon| {
        let client = client.clone();
        let api_key = curseforge_api_key.clone();
        let flavor = flavor.clone();
        let version_type = version_type.clone();
        async move {
            match addon.source.as_str() {
                "curseforge" => {
                    let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) else {
                        return UpdateCheckResult::error(
                            addon.id,
                            "CurseForge 插件需要在设置中填写 API Key。",
                        );
                    };
                    let version_type_id = match version_type {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            return UpdateCheckResult::error(
                                addon.id,
                                "未找到与当前客户端匹配的 CurseForge 游戏版本。",
                            )
                        }
                        Err(error) => return UpdateCheckResult::error(addon.id, error),
                    };
                    check_curseforge(&client, addon, &api_key, version_type_id).await
                }
                "wowinterface" => check_wowinterface(&client, addon, &flavor).await,
                _ => UpdateCheckResult::untracked(addon.id),
            }
        }
    }))
    .buffer_unordered(6)
    .collect::<Vec<_>>()
    .await;

    Ok(results)
}

async fn find_curseforge_version_type(
    client: &Client,
    api_key: &str,
    flavor: &str,
) -> Result<Option<u64>, String> {
    let response = client
        .get(format!(
            "{CURSEFORGE_API}/games/{WOW_GAME_ID}/version-types"
        ))
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|error| format!("连接 CurseForge 失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("CurseForge 拒绝了请求：{error}"))?
        .json::<ApiResponse<Vec<GameVersionType>>>()
        .await
        .map_err(|error| format!("无法解析 CurseForge 版本类型：{error}"))?;

    let wanted = flavor_tokens(flavor);
    Ok(response
        .data
        .into_iter()
        .find(|item| matches_curseforge_flavor(item, flavor, &wanted))
        .map(|item| item.id))
}

async fn check_curseforge(
    client: &Client,
    addon: AddonInfo,
    api_key: &str,
    version_type_id: u64,
) -> UpdateCheckResult {
    let Some(project_id) = addon.source_id.as_deref() else {
        return UpdateCheckResult::untracked(addon.id);
    };
    let mut request = client
        .get(format!(
            "{CURSEFORGE_API}/mods/{project_id}/files?pageSize=50"
        ))
        .header("x-api-key", api_key);
    request = request.query(&[("gameVersionTypeId", version_type_id)]);
    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            return UpdateCheckResult::error(addon.id, format!("连接 CurseForge 失败：{error}"))
        }
    };
    let response = match response.error_for_status() {
        Ok(response) => response,
        Err(error) => {
            return UpdateCheckResult::error(addon.id, format!("CurseForge 请求失败：{error}"))
        }
    };
    let payload = match response.json::<ApiResponse<Vec<CurseFile>>>().await {
        Ok(payload) => payload,
        Err(error) => {
            return UpdateCheckResult::error(
                addon.id,
                format!("CurseForge 返回了无法识别的数据：{error}"),
            )
        }
    };

    let Some(file) = payload
        .data
        .into_iter()
        .filter(|file| file.release_type == 1)
        .max_by(|left, right| left.file_date.cmp(&right.file_date))
    else {
        return UpdateCheckResult::error(addon.id, "没有找到适用于此游戏版本的正式发布。");
    };

    let version = display_version(&file);
    let status = if is_remote_newer(&addon.version, &version) {
        "update"
    } else {
        "current"
    };
    UpdateCheckResult {
        addon_id: addon.id,
        status: status.into(),
        latest_version: Some(version),
        latest_file_id: Some(file.id.to_string()),
        download_url: file.download_url,
        website_url: Some(format!(
            "https://www.curseforge.com/wow/addons/{project_id}"
        )),
        error: None,
    }
}

async fn check_wowinterface(client: &Client, addon: AddonInfo, flavor: &str) -> UpdateCheckResult {
    let Some(project_id) = addon.source_id.as_deref() else {
        return UpdateCheckResult::untracked(addon.id);
    };
    let url = format!("https://api.mmoui.com/game/WOW/filedetails/{project_id}.json");
    let payload = match client.get(url).send().await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => match response.json::<Value>().await {
                Ok(payload) => payload,
                Err(error) => {
                    return UpdateCheckResult::error(
                        addon.id,
                        format!("WoWInterface 返回了无法识别的数据：{error}"),
                    )
                }
            },
            Err(error) => {
                return UpdateCheckResult::error(
                    addon.id,
                    format!("WoWInterface 请求失败：{error}"),
                )
            }
        },
        Err(error) => {
            return UpdateCheckResult::error(addon.id, format!("连接 WoWInterface 失败：{error}"))
        }
    };

    let details = payload
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(&payload);
    let compatibility = json_string(
        details,
        &[
            "Compatibility",
            "compatibility",
            "GameVersion",
            "gameVersion",
            "UICompatibility",
        ],
    );
    if !wowinterface_supports_flavor(compatibility.as_deref(), flavor) {
        return UpdateCheckResult::error(
            addon.id,
            "WoWInterface 未确认此文件与当前客户端版本兼容，已跳过更新。",
        );
    }
    let version = json_string(
        details,
        &[
            "Version",
            "version",
            "UIVersion",
            "uiversion",
            "UIFileVersion",
        ],
    )
    .unwrap_or_else(|| "未知".into());
    let download_url = json_string(
        details,
        &[
            "Download",
            "download",
            "downloadUri",
            "UIFileDownloadURL",
            "UIDownload",
        ],
    )
    .or_else(|| {
        Some(format!(
            "https://cdn.wowinterface.com/downloads/getfile.php?id={project_id}"
        ))
    });
    let file_id = json_string(details, &["UID", "uid", "Id", "id", "UIFileID"]);
    let status = if version != "未知" && is_remote_newer(&addon.version, &version) {
        "update"
    } else {
        "current"
    };

    UpdateCheckResult {
        addon_id: addon.id,
        status: status.into(),
        latest_version: Some(version),
        latest_file_id: file_id,
        download_url,
        website_url: Some(format!(
            "https://www.wowinterface.com/downloads/info{project_id}.html"
        )),
        error: None,
    }
}

fn flavor_tokens(flavor: &str) -> Vec<&'static str> {
    match flavor {
        "retail" => vec!["retail"],
        "ptr" => vec!["retail", "ptr"],
        "beta" => vec!["retail", "beta"],
        "classic_era" => vec!["classic", "era"],
        "classic_anniversary" => vec!["classic", "anniversary"],
        "classic_titan" => vec!["classic", "titan"],
        "classic_ptr" => vec!["classic", "ptr"],
        "classic" => vec!["classic"],
        _ => vec!["retail"],
    }
}

fn matches_curseforge_flavor(
    version_type: &GameVersionType,
    flavor: &str,
    wanted: &[&str],
) -> bool {
    let haystack = format!("{} {}", version_type.name, version_type.slug).to_lowercase();
    if flavor == "classic" {
        // `classic` is intentionally the generic progression client. Do not
        // let API response ordering select Era, PTR, or Anniversary by chance.
        return haystack.contains("classic")
            && !["era", "anniversary", "ptr", "beta"]
                .iter()
                .any(|token| haystack.contains(token));
    }
    wanted.iter().all(|token| haystack.contains(token))
}

fn wowinterface_supports_flavor(compatibility: Option<&str>, flavor: &str) -> bool {
    let Some(compatibility) = compatibility else {
        // WoWInterface's normal WoW endpoint is Retail-only unless a response
        // explicitly advertises another flavor.
        return flavor == "retail";
    };
    let value = compatibility.to_lowercase();
    flavor_tokens(flavor)
        .iter()
        .all(|token| value.contains(token))
}

fn display_version(file: &CurseFile) -> String {
    let display_name = file
        .display_name
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    display_name
        .and_then(extract_release_version)
        .or_else(|| extract_release_version(&file.file_name))
        .or_else(|| display_name.map(|value| value.trim().to_string()))
        .unwrap_or_else(|| trim_archive_extension(&file.file_name).to_string())
}

fn extract_release_version(value: &str) -> Option<String> {
    let characters: Vec<char> = trim_archive_extension(value).chars().collect();
    let mut index = 0;
    let mut last_candidate = None;

    while index < characters.len() {
        if !characters[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        if index > 0 && characters[index - 1].is_ascii_digit() {
            index += 1;
            continue;
        }

        let start = if index > 0
            && matches!(characters[index - 1], 'v' | 'V')
            && (index == 1 || !characters[index - 2].is_ascii_alphanumeric())
        {
            index - 1
        } else {
            index
        };
        let mut end = index;
        while end < characters.len()
            && (characters[end].is_ascii_alphanumeric()
                || matches!(characters[end], '.' | '-' | '_'))
        {
            end += 1;
        }
        last_candidate = Some(characters[start..end].iter().collect());
        index = end;
    }

    last_candidate
}

fn trim_archive_extension(value: &str) -> &str {
    value.trim_end_matches(".zip").trim_end_matches(".ZIP")
}

fn json_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .or_else(|| value.as_u64().map(|number| number.to_string()))
        })
    })
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameVersionType {
    id: u64,
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseFile {
    id: u64,
    display_name: Option<String>,
    file_name: String,
    release_type: u8,
    file_date: String,
    download_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{display_version, CurseFile};

    #[test]
    fn extracts_a_comparable_version_from_curseforge_release_names() {
        let details_release = CurseFile {
            id: 1,
            display_name: Some("Details! Damage Meter (Retail) (11.2.0.14010)".into()),
            file_name: "Details-11.2.0.14010.zip".into(),
            release_type: 1,
            file_date: "2026-07-29T00:00:00Z".into(),
            download_url: Some("https://example.invalid/details.zip".into()),
        };
        let plater_release = CurseFile {
            id: 2,
            display_name: None,
            file_name: "Plater-v612-Retail.zip".into(),
            release_type: 1,
            file_date: "2026-07-29T00:00:00Z".into(),
            download_url: Some("https://example.invalid/plater.zip".into()),
        };

        assert_eq!(display_version(&details_release), "11.2.0.14010");
        assert_eq!(display_version(&plater_release), "v612-Retail");
    }
}
