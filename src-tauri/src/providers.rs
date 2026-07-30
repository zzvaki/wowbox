use crate::{
    models::{AddonAuthor, AddonDetails, AddonInfo, UpdateCheckResult},
    provider_config::curseforge_api_key,
    version::is_remote_newer,
};
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const WOW_GAME_ID: u64 = 1;

pub async fn check_updates(
    addons: Vec<AddonInfo>,
    flavor: String,
    provider: String,
    user_curseforge_api_key: Option<String>,
) -> Result<Vec<UpdateCheckResult>, String> {
    match provider.as_str() {
        "curseforge" => {}
        "wowinterface" => {
            return Err("WoWInterface 数据源正在开发中；一期请使用 CurseForge。".into())
        }
        _ => return Err("不支持的数据源。".into()),
    }
    let api_key = curseforge_api_key(user_curseforge_api_key.as_deref()).ok_or_else(|| {
        "未配置 CurseForge API Key。请填写个人 Key，或在构建时提供默认 Key。".to_string()
    })?;
    let client = Client::builder()
        .user_agent(format!("WowBox/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("无法初始化网络客户端：{error}"))?;

    // Never fall back to CurseForge's unfiltered files endpoint: doing so could
    // install a Retail package into a Classic client when version-type lookup
    // fails or the selected game flavor is unknown.
    let version_type = find_curseforge_version_type(&client, &api_key, &flavor).await;

    let results = stream::iter(addons.into_iter().map(|addon| {
        let client = client.clone();
        let api_key = api_key.clone();
        let version_type = version_type.clone();
        async move {
            if addon.source == "wowinterface" {
                return UpdateCheckResult::untracked(addon.id);
            }
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
    let project = match resolve_curse_project(client, &addon, api_key, Some(version_type_id)).await
    {
        Ok(Some(project)) => project,
        Ok(None) => return UpdateCheckResult::untracked(addon.id),
        Err(error) => return UpdateCheckResult::error(addon.id, error),
    };
    let project_id = project.id;
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
    let author = joined_authors(&project.authors);
    let website_url = project.links.website_url.clone().or_else(|| {
        Some(format!(
            "https://www.curseforge.com/wow/addons/{}",
            project.slug
        ))
    });
    UpdateCheckResult {
        addon_id: addon.id,
        status: status.into(),
        title: Some(project.name),
        author,
        summary: project.summary.filter(|summary| !summary.trim().is_empty()),
        source_id: Some(project_id.to_string()),
        latest_version: Some(version),
        latest_file_id: Some(file.id.to_string()),
        download_url: file.download_url,
        website_url,
        error: None,
    }
}

pub async fn fetch_addon_details(
    addon: AddonInfo,
    flavor: String,
    user_curseforge_api_key: Option<String>,
) -> Result<AddonDetails, String> {
    if addon.source == "wowinterface" {
        return Err("当前插件来自 WoWInterface，无法查询 CurseForge 详情。".into());
    }
    let api_key = curseforge_api_key(user_curseforge_api_key.as_deref())
        .ok_or_else(|| "未配置 CurseForge API Key。请在设置中填写个人 Key。".to_string())?;
    let client = Client::builder()
        .user_agent(format!("WowBox/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("无法初始化网络客户端：{error}"))?;
    let needs_search = addon.source != "curseforge" || addon.source_id.is_none();
    let version_type_id = if needs_search {
        Some(
            find_curseforge_version_type(&client, &api_key, &flavor)
                .await?
                .ok_or_else(|| "未找到与当前客户端匹配的 CurseForge 游戏版本。".to_string())?,
        )
    } else {
        None
    };
    let project = resolve_curse_project(&client, &addon, &api_key, version_type_id)
        .await?
        .ok_or_else(|| "未能在 CurseForge 中准确匹配这个插件。".to_string())?;
    // Description is supplemental. Keep the structured project details usable
    // when CurseForge omits or temporarily fails this separate endpoint.
    let description = fetch_project_description(&client, &api_key, project.id)
        .await
        .unwrap_or_default();
    Ok(addon_details(project, description))
}

async fn resolve_curse_project(
    client: &Client,
    addon: &AddonInfo,
    api_key: &str,
    version_type_id: Option<u64>,
) -> Result<Option<CurseProject>, String> {
    if let Some(project_id) = addon
        .source_id
        .as_deref()
        .filter(|_| addon.source == "curseforge")
    {
        let project_id = project_id
            .parse::<u64>()
            .map_err(|_| "CurseForge Project ID 格式无效。".to_string())?;
        return fetch_curse_project(client, api_key, project_id)
            .await
            .map(Some);
    }
    let version_type_id =
        version_type_id.ok_or_else(|| "搜索插件时缺少 CurseForge 游戏版本。".to_string())?;
    search_curse_project(client, addon, api_key, version_type_id).await
}

async fn fetch_curse_project(
    client: &Client,
    api_key: &str,
    project_id: u64,
) -> Result<CurseProject, String> {
    client
        .get(format!("{CURSEFORGE_API}/mods/{project_id}"))
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|error| format!("连接 CurseForge 插件信息失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("CurseForge 插件信息请求失败：{error}"))?
        .json::<ApiResponse<CurseProject>>()
        .await
        .map(|payload| payload.data)
        .map_err(|error| format!("CurseForge 插件信息无法解析：{error}"))
}

async fn search_curse_project(
    client: &Client,
    addon: &AddonInfo,
    api_key: &str,
    version_type_id: u64,
) -> Result<Option<CurseProject>, String> {
    let mut search_terms = vec![addon.folder_name.trim(), addon.title.trim()];
    search_terms.dedup();
    let mut candidates = Vec::new();

    for search_term in search_terms.into_iter().filter(|term| !term.is_empty()) {
        let response = client
            .get(format!("{CURSEFORGE_API}/mods/search"))
            .header("x-api-key", api_key)
            .query(&[
                ("gameId", WOW_GAME_ID.to_string()),
                ("gameVersionTypeId", version_type_id.to_string()),
                ("searchFilter", search_term.to_string()),
                ("sortField", "2".to_string()),
                ("sortOrder", "desc".to_string()),
                ("pageSize", "10".to_string()),
            ])
            .send()
            .await
            .map_err(|error| format!("连接 CurseForge 搜索接口失败：{error}"))?
            .error_for_status()
            .map_err(|error| format!("CurseForge 搜索请求失败：{error}"))?
            .json::<SearchModsResponse>()
            .await
            .map_err(|error| format!("CurseForge 搜索结果无法解析：{error}"))?;
        candidates.extend(response.data);
    }

    candidates.sort_by_key(|project| project.id);
    candidates.dedup_by_key(|project| project.id);
    let matches = candidates
        .into_iter()
        .filter_map(|project| {
            let score = project_match_score(addon, &project);
            (score > 0).then_some((score, project))
        })
        .collect::<Vec<_>>();
    let Some(best_score) = matches.iter().map(|(score, _)| *score).max() else {
        return Ok(None);
    };
    let mut best_matches = matches
        .into_iter()
        .filter(|(score, _)| *score == best_score);
    let Some((_, project)) = best_matches.next() else {
        return Ok(None);
    };
    if best_matches.next().is_some() {
        return Ok(None);
    }
    Ok(Some(project))
}

async fn fetch_project_description(
    client: &Client,
    api_key: &str,
    project_id: u64,
) -> Result<String, String> {
    client
        .get(format!("{CURSEFORGE_API}/mods/{project_id}/description"))
        .header("x-api-key", api_key)
        .query(&[("stripped", true)])
        .send()
        .await
        .map_err(|error| format!("连接 CurseForge 插件描述接口失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("CurseForge 插件描述请求失败：{error}"))?
        .json::<ApiResponse<String>>()
        .await
        .map(|payload| payload.data)
        .map_err(|error| format!("CurseForge 插件描述无法解析：{error}"))
}

fn project_match_score(addon: &AddonInfo, project: &CurseProject) -> u8 {
    let local_title = normalized_identifier(&addon.title);
    let local_folder = normalized_identifier(&addon.folder_name);
    let remote_name = normalized_identifier(&project.name);
    let remote_slug = normalized_identifier(&project.slug);
    let exact_name = !local_title.is_empty() && local_title == remote_name;
    let exact_folder =
        !local_folder.is_empty() && (local_folder == remote_slug || local_folder == remote_name);
    if !exact_name && !exact_folder {
        return 0;
    }

    let local_author = normalized_identifier(&addon.author);
    let author_match = !local_author.is_empty()
        && project
            .authors
            .iter()
            .any(|author| normalized_identifier(&author.name) == local_author);
    u8::from(exact_name) * 4 + u8::from(exact_folder) * 4 + u8::from(author_match)
}

fn normalized_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn joined_authors(authors: &[CurseAuthor]) -> Option<String> {
    let names: Vec<&str> = authors
        .iter()
        .map(|author| author.name.trim())
        .filter(|name| !name.is_empty())
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

fn addon_details(project: CurseProject, description: String) -> AddonDetails {
    let website_url = project.links.website_url.clone().or_else(|| {
        Some(format!(
            "https://www.curseforge.com/wow/addons/{}",
            project.slug
        ))
    });
    AddonDetails {
        project_id: project.id.to_string(),
        name: project.name,
        slug: project.slug,
        summary: project.summary.unwrap_or_default(),
        description,
        authors: project
            .authors
            .into_iter()
            .map(|author| AddonAuthor {
                name: author.name,
                url: author.url,
            })
            .collect(),
        categories: project
            .categories
            .into_iter()
            .map(|category| category.name)
            .collect(),
        download_count: project.download_count,
        thumbs_up_count: project.thumbs_up_count,
        rating: project.rating,
        website_url,
        wiki_url: project.links.wiki_url,
        issues_url: project.links.issues_url,
        source_url: project.links.source_url,
        date_created: project.date_created,
        date_modified: project.date_modified,
        date_released: project.date_released,
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

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

type SearchModsResponse = ApiResponse<Vec<CurseProject>>;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseProject {
    id: u64,
    name: String,
    slug: String,
    summary: Option<String>,
    #[serde(default)]
    links: CurseProjectLinks,
    #[serde(default)]
    authors: Vec<CurseAuthor>,
    #[serde(default)]
    categories: Vec<CurseCategory>,
    #[serde(default)]
    download_count: u64,
    #[serde(default)]
    thumbs_up_count: u64,
    rating: Option<f64>,
    date_created: Option<String>,
    date_modified: Option<String>,
    date_released: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseProjectLinks {
    website_url: Option<String>,
    wiki_url: Option<String>,
    issues_url: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseAuthor {
    name: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseCategory {
    name: String,
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
