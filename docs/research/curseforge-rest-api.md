# CurseForge REST API 调研

调研日期：2026-07-30

范围：仅核对 CurseForge 官方 REST 文档、官方数据模型及官方第三方 API 条款。本文不包含任何真实 API Key。

## 结论摘要

- REST 基地址为 `https://api.curseforge.com`，本文涉及的接口都位于 `/v1` 下。请求通过 HTTP Header `x-api-key` 认证，通常同时发送 `Accept: application/json`。[官方：Base URL、Authentication](https://docs.curseforge.com/rest-api/?shell#accessing-the-service)
- 插件搜索应先用 `GET /v1/mods/search` 找到 CurseForge `modId`；已知 `modId` 后用 `GET /v1/mods/{modId}` 读取结构化详情，再用 `GET /v1/mods/{modId}/description` 获取完整 HTML 描述。[官方：Search Mods](https://docs.curseforge.com/rest-api/?shell#search-mods) · [Get Mod](https://docs.curseforge.com/rest-api/?shell#get-mod) · [Get Mod Description](https://docs.curseforge.com/rest-api/?shell#get-mod-description)
- 搜索和文件列表的分页 `pageSize` 默认及最大均为 50，`index` 从 0 开始，且 `index + pageSize <= 10,000`。[官方：Pagination Limits](https://docs.curseforge.com/rest-api/?shell#pagination-limits)
- 官方 REST 文档未公布固定的“每分钟请求数”，接口响应表也没有把 `429` 列为标准响应。官方条款只说明存在由 Overwolf 自行决定、可随时更新的配额，超额后可能要求另签许可或拒绝继续访问。[官方：第三方 API 条款 §2.3](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)
- **不可把开发者 API Key 分发给桌面客户端用户。** 官方条款称 Key 不可转让、不可与第三方共享；将 Key 编译进客户端并不能保密，也会把 Key 随应用分发。[官方：第三方 API 条款 §2.2](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)
- 官方条款还禁止保存/缓存 API 数据，并禁止使用 API 构建与 CurseForge 直接或间接竞争的产品。这会直接影响本项目的产品形态和本地缓存设计，上线前应先取得 CurseForge 的书面授权或确认许可范围。[官方：第三方 API 条款 §3.1](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)

## 通用请求约定

| 项目 | 契约 |
| --- | --- |
| Base URL | `https://api.curseforge.com` |
| 版本前缀 | `/v1` |
| 认证 | `x-api-key: <API_KEY>` |
| 接受类型 | `Accept: application/json` |
| 分页起点 | `index=0` |
| 单页上限 | `pageSize=50` |
| 总检索窗口 | `index + pageSize <= 10,000` |

来源：[Base URL](https://docs.curseforge.com/rest-api/?shell#base-url) · [Authentication](https://docs.curseforge.com/rest-api/?shell#authentication) · [Pagination Limits](https://docs.curseforge.com/rest-api/?shell#pagination-limits)

官方还说明，除非端点另有声明，所有 `int32` 响应都应视为无符号整数。[官方：Notes](https://docs.curseforge.com/rest-api/?shell#notes)

## 1. Search Mods

### 请求

```http
GET /v1/mods/search
Accept: application/json
x-api-key: <API_KEY>
```

`gameId` 是唯一必填查询参数。可选参数如下：

| 参数 | 类型 | 用途/限制 |
| --- | --- | --- |
| `gameId` | int32 | 必填，按游戏 ID 过滤 |
| `classId` | int32 | 按 section/class 过滤，可通过 Categories 接口发现 |
| `categoryId` | int32 | 单一分类 |
| `categoryIds` | string | 分类列表，覆盖 `categoryId`；最多 10 个 |
| `gameVersion` | string | 单一游戏版本 |
| `gameVersions` | string | 游戏版本列表，覆盖 `gameVersion`；官方详细说明处写明最多 4 个，但限制原文误写成了 “category ids”，应视为文档措辞错误并避免依赖其余未说明的编码细节 |
| `searchFilter` | string | 在插件名称和作者中做自由文本搜索 |
| `sortField` | enum | 排序字段，见下表 |
| `sortOrder` | `asc` / `desc` | 排序方向 |
| `modLoaderType` | enum | 与 `gameVersion` 配合使用 |
| `modLoaderTypes` | string | mod loader 列表，覆盖 `modLoaderType`；最多 5 个 |
| `gameVersionTypeId` | int32 | 只返回包含该版本类型文件的插件 |
| `authorId` | int32 | 按项目成员过滤 |
| `primaryAuthorId` | int32 | 按项目所有者过滤 |
| `slug` | string | 按 slug 过滤；官方称与 `classId` 一起使用时可得到唯一结果 |
| `index` | int32 | 从 0 开始的首条索引 |
| `pageSize` | int32 | 默认/最大 50 |

`sortField` 枚举：

| 值 | 含义 | 值 | 含义 |
| --- | --- | --- | --- |
| 1 | Featured | 7 | Category |
| 2 | Popularity | 8 | GameVersion |
| 3 | LastUpdated | 9 | EarlyAccess |
| 4 | Name | 10 | FeaturedReleased |
| 5 | Author | 11 | ReleasedDate |
| 6 | TotalDownloads | 12 | Rating |

来源：[Search Mods 参数](https://docs.curseforge.com/rest-api/?shell#search-mods) · [ModsSearchSortField](https://docs.curseforge.com/rest-api/?shell#modssearchsortfield)

### 响应

```json
{
  "data": [
    {
      "id": 0,
      "gameId": 0,
      "name": "string",
      "slug": "string",
      "summary": "string"
    }
  ],
  "pagination": {
    "index": 0,
    "pageSize": 0,
    "resultCount": 0,
    "totalCount": 0
  }
}
```

`data` 中每项都是完整的 `Mod` 数据模型，不只是搜索摘要。除示例字段外，还包括：

- `links`：`websiteUrl`、`wikiUrl`、`issuesUrl`、`sourceUrl`
- `status`、`downloadCount`、`isFeatured`
- `primaryCategoryId`、`categories`、`classId`
- `authors`
- `logo`、`screenshots`
- `mainFileId`
- `latestFiles`、`latestFilesIndexes`、`latestEarlyAccessFilesIndexes`
- `dateCreated`、`dateModified`、`dateReleased`
- `allowModDistribution`
- `gamePopularityRank`、`isAvailable`、`thumbsUpCount`、`rating`

来源：[Search Mods Response](https://docs.curseforge.com/rest-api/?shell#search-mods-response) · [Mod schema](https://docs.curseforge.com/rest-api/?shell#mod) · [Pagination schema](https://docs.curseforge.com/rest-api/?shell#pagination)

响应状态：官方列出 `200`、`400`、`500`。[官方：Search Mods Responses](https://docs.curseforge.com/rest-api/?shell#search-mods)

### 用于本项目的匹配注意事项

- `searchFilter` 搜索范围是“插件名称和作者”，不是精确匹配；不能直接把第一条结果视为正确插件。
- 更稳妥的匹配顺序是：TOC 内已存在的 CurseForge `modId` → 已知 `slug + classId` → 搜索候选后比较规范化名称/作者并让用户确认。
- `gameVersionTypeId` 比仅按文本 `gameVersion` 更适合区分同一游戏的多个客户端分支，但版本类型 ID 应通过官方 Games Version Types 接口发现，不应依赖未经验证的固定映射。[官方：Get Version Types](https://docs.curseforge.com/rest-api/?shell#get-version-types)

以上三点中的匹配顺序是基于官方参数语义给出的实现建议，不是 CurseForge 官方规定的匹配算法。

## 2. Get Mod

### 请求

```http
GET /v1/mods/{modId}
Accept: application/json
x-api-key: <API_KEY>
```

唯一参数：

| 参数 | 位置 | 类型 | 必填 |
| --- | --- | --- | --- |
| `modId` | path | int32 | 是 |

来源：[Get Mod](https://docs.curseforge.com/rest-api/?shell#get-mod)

### 响应

响应包装为：

```json
{
  "data": {
    "id": 0,
    "gameId": 0,
    "name": "string",
    "slug": "string",
    "summary": "string"
  }
}
```

`data` 的完整字段就是上一节列出的 `Mod` 模型。适合详情页使用的重点字段为：

- 标识：`id`、`gameId`、`name`、`slug`
- 简介与链接：`summary`、`links`
- 作者与分类：`authors`、`categories`
- 视觉素材：`logo`、`screenshots`
- 热度：`downloadCount`、`gamePopularityRank`、`thumbsUpCount`、`rating`
- 时间：`dateCreated`、`dateModified`、`dateReleased`
- 文件入口：`mainFileId`、`latestFiles`、`latestFilesIndexes`
- 可用/分发状态：`isAvailable`、`allowModDistribution`

来源：[Get Mod](https://docs.curseforge.com/rest-api/?shell#get-mod) · [Mod schema](https://docs.curseforge.com/rest-api/?shell#mod)

响应状态：官方列出 `200`、`404`、`500`。[官方：Get Mod Responses](https://docs.curseforge.com/rest-api/?shell#get-mod)

## 3. Get Mod Description

### 请求

```http
GET /v1/mods/{modId}/description
Accept: application/json
x-api-key: <API_KEY>
```

参数：

| 参数 | 位置 | 类型 | 必填 | 官方描述 |
| --- | --- | --- | --- | --- |
| `modId` | path | int32 | 是 | mod id |
| `raw` | query | boolean | 否 | 官方未补充语义 |
| `stripped` | query | boolean | 否 | 官方未补充语义 |
| `markup` | query | boolean | 否 | 官方未补充语义 |

官方将此接口描述为“以 HTML 格式获取插件完整描述”。响应为：

```json
{
  "data": "<p>HTML description</p>"
}
```

来源：[Get Mod Description](https://docs.curseforge.com/rest-api/?shell#get-mod-description) · [String Response](https://docs.curseforge.com/rest-api/?shell#string-response)

响应状态：官方列出 `200`、`404`、`500`。[官方：Get Mod Description Responses](https://docs.curseforge.com/rest-api/?shell#get-mod-description)

### UI 安全建议

返回内容是远端 HTML。详情页不应把它未经处理直接交给 Vue `v-html`；应在原生端或前端使用严格白名单清洗，移除脚本、事件属性、危险 URL 协议和可执行嵌入内容。外链应限制为允许的 `http/https` 并通过系统浏览器打开。

这是基于响应格式的安全实现建议；官方端点文档只保证 `data` 为 HTML 字符串，并未声明内容已经过适合嵌入任意 WebView 的安全清洗。

## 4. Get Mod Files

### 请求

```http
GET /v1/mods/{modId}/files
Accept: application/json
x-api-key: <API_KEY>
```

参数：

| 参数 | 位置 | 类型 | 必填 | 用途 |
| --- | --- | --- | --- | --- |
| `modId` | path | int32 | 是 | 文件所属插件 |
| `gameVersion` | query | string | 否 | 按游戏版本字符串过滤 |
| `modLoaderType` | query | enum | 否 | 按 mod loader 过滤 |
| `gameVersionTypeId` | query | int32 | 否 | 按游戏版本类型过滤 |
| `index` | query | int32 | 否 | 从 0 开始的首条索引 |
| `pageSize` | query | int32 | 否 | 默认/最大 50 |

`modLoaderType` 官方枚举：`0=Any`、`1=Forge`、`2=Cauldron`、`3=LiteLoader`、`4=Fabric`、`5=Quilt`、`6=NeoForge`。

来源：[Get Mod Files](https://docs.curseforge.com/rest-api/?shell#get-mod-files) · [ModLoaderType](https://docs.curseforge.com/rest-api/?shell#modloadertype)

### 响应

```json
{
  "data": [
    {
      "id": 0,
      "modId": 0,
      "displayName": "string",
      "fileName": "string",
      "fileDate": "2019-08-24T14:15:22Z",
      "downloadUrl": "string",
      "gameVersions": []
    }
  ],
  "pagination": {
    "index": 0,
    "pageSize": 0,
    "resultCount": 0,
    "totalCount": 0
  }
}
```

每个 `File` 的完整重点字段：

- 标识/状态：`id`、`gameId`、`modId`、`isAvailable`、`fileStatus`
- 展示/下载：`displayName`、`fileName`、`downloadUrl`
- 发布信息：`releaseType`（`1=Release`、`2=Beta`、`3=Alpha`）、`fileDate`
- 文件信息：`hashes`、`fileLength`、`fileSizeOnDisk`、`fileFingerprint`
- 兼容性：`gameVersions`、`sortableGameVersions`
- 依赖：`dependencies`
- 其他关系：`parentProjectFileId`、`alternateFileId`、`serverPackFileId`
- Early Access：`isEarlyAccessContent`、`earlyAccessEndDate`
- 文件模块：`modules`

来源：[Get Mod Files](https://docs.curseforge.com/rest-api/?shell#get-mod-files) · [File schema](https://docs.curseforge.com/rest-api/?shell#file) · [FileReleaseType](https://docs.curseforge.com/rest-api/?shell#filereleasetype)

响应状态：官方列出 `200`、`404`、`500`。[官方：Get Mod Files Responses](https://docs.curseforge.com/rest-api/?shell#get-mod-files)

## 5. Get Mod File

### 请求

```http
GET /v1/mods/{modId}/files/{fileId}
Accept: application/json
x-api-key: <API_KEY>
```

参数：

| 参数 | 位置 | 类型 | 必填 |
| --- | --- | --- | --- |
| `modId` | path | int32 | 是 |
| `fileId` | path | int32 | 是 |

来源：[Get Mod File](https://docs.curseforge.com/rest-api/?shell#get-mod-file)

### 响应

```json
{
  "data": {
    "id": 0,
    "modId": 0,
    "displayName": "string",
    "fileName": "string",
    "downloadUrl": "string"
  }
}
```

`data` 是完整 `File` 模型，字段与上一节相同；该接口不带 `pagination`。响应状态：官方列出 `200`、`404`、`500`。

来源：[Get Mod File](https://docs.curseforge.com/rest-api/?shell#get-mod-file) · [Get Mod File Response](https://docs.curseforge.com/rest-api/?shell#get-mod-file-response) · [File schema](https://docs.curseforge.com/rest-api/?shell#file)

## 分页和请求调度

搜索与文件列表都应根据响应中的：

- `pagination.index`
- `pagination.pageSize`
- `pagination.resultCount`
- `pagination.totalCount`

决定是否继续下一页。下一页索引通常为 `index + resultCount`；当 `resultCount == 0`、已达到 `totalCount`，或下一次请求会超过 10,000 检索窗口时停止。这是依据官方分页字段给出的实现方式。[官方：Pagination schema](https://docs.curseforge.com/rest-api/?shell#pagination)

官方没有给出固定数字限流，因此实现侧建议：

- 复用项目详情和版本类型的同一次刷新结果，避免同屏重复请求；
- 限制并发，扫描大量插件时分批查询；
- 对网络错误和 `5xx` 做带抖动的指数退避；
- 即便当前响应表未列出 `429`，仍防御性处理它，并在服务端返回 `Retry-After` 时遵守该值；
- 不把“重试缓存”演变为持久化 API 数据缓存，除非已获得官方许可。

以上调度策略是防御性实现建议；可确认的官方事实只有分页上限和条款中的动态配额。

## 分发与合规注意事项

### API Key

官方条款规定：

- 每位开发者取得唯一 API Key；
- Key 不可转让；
- 不得向第三方共享，受保密义务约束的员工例外。

因此，对于公开分发的 Tauri 桌面应用：

- **不要**把开发者 Key 放入前端代码、`VITE_*` 环境变量、资源文件或编译后的原生二进制；
- “在 UI 中不展示”不等于保密，用户仍可从二进制或请求中提取 Key；
- 合规路径是让用户输入其自己的合法 Key，或先取得 CurseForge 对该分发方式的明确书面授权；如采用服务端代理，也必须确认代理共享调用是否得到允许。

来源：[第三方 API 条款 §2](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)

### 数据缓存与产品用途

官方条款 §3.1 写明不得保存或缓存通过 API/SDK 获得的数据，并禁止利用 API/SDK 或其材料构建与 CurseForge 直接或间接竞争的产品。本项目是插件管理与更新工具，因此在继续公开分发前需要先完成许可确认。

来源：[第三方 API 条款 §3](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)

### 项目分发开关

项目作者可以关闭第三方分发。官方说明关闭后，第三方服务、网站或客户端将无法通过第三方 API 访问其项目或文件；API 会优先尊重该开关。`Mod.allowModDistribution` 字段用于表达插件是否允许分发。

来源：[Project Distribution Toggle](https://support.curseforge.com/support/solutions/articles/9000207877-project-distribution-toggle) · [Mod schema: allowModDistribution](https://docs.curseforge.com/rest-api/?shell#mod)

实现上应把 `allowModDistribution != true` 视为不可由本工具下载/更新，并向用户解释原因；这是基于官方字段与分发开关规则给出的保守实现建议。

## 建议的详情页数据流

1. 本地 TOC 若有可信 `modId`，直接请求 Get Mod；否则调用 Search Mods 产生候选。
2. 用户打开详情时请求 Get Mod，展示结构化元数据。
3. 同时按需请求 Get Mod Description，并在清洗 HTML 后展示正文。
4. 展示版本历史时请求 Get Mod Files，使用 `gameVersionTypeId` 或 `gameVersion` 限定当前 WoW 客户端分支。
5. 用户选中具体版本时，可调用 Get Mod File 再确认文件状态、兼容版本、哈希和下载地址。
6. 下载前检查 `isAvailable`、`allowModDistribution`、目标文件 `isAvailable`，并验证下载 URL 和文件哈希。

上述流程是由官方接口能力推导的本项目实现建议，不是官方规定的客户端流程。

## 官方来源索引

- [CurseForge for Studios REST API](https://docs.curseforge.com/rest-api/)
- [Search Mods](https://docs.curseforge.com/rest-api/?shell#search-mods)
- [Get Mod](https://docs.curseforge.com/rest-api/?shell#get-mod)
- [Get Mod Description](https://docs.curseforge.com/rest-api/?shell#get-mod-description)
- [Get Mod Files](https://docs.curseforge.com/rest-api/?shell#get-mod-files)
- [Get Mod File](https://docs.curseforge.com/rest-api/?shell#get-mod-file)
- [Mod schema](https://docs.curseforge.com/rest-api/?shell#mod)
- [File schema](https://docs.curseforge.com/rest-api/?shell#file)
- [Pagination schema](https://docs.curseforge.com/rest-api/?shell#pagination)
- [CurseForge 3rd Party API Terms and Conditions](https://support.curseforge.com/support/solutions/articles/9000207405-curseforge-3rd-party-api-terms-and-conditions)
- [Project Distribution Toggle](https://support.curseforge.com/support/solutions/articles/9000207877-project-distribution-toggle)
