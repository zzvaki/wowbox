# WowBox

WowBox 是一个专注于“扫描、识别、更新”的跨平台《魔兽世界》插件管理器。前端使用 Vite、Vue 3 和 Naive UI，桌面与本地能力由 Tauri 2 / Rust 提供，不需要部署服务端。

## 一期能力

- 自动检测 macOS / Windows 上的 World of Warcraft 目录
- 支持正式服、经典进度服、经典旧世、周年纪念服、PTR 和 Beta 等多个产品目录
- 解析插件 `.toc` 文件中的中文标题、描述、作者、版本、Interface 和更新源 ID
- 使用 `X-Curse-Project-ID` 合并一个插件包包含的多个文件夹
- 通过 CurseForge REST API 查询插件信息与可用更新
- 在 UI 中更新单个插件或全部插件
- 更新前自动备份旧目录，安装失败时自动回滚
- 拒绝 ZIP 路径穿越，限制下载包大小为 200 MB

## 本地开发

环境要求：

- Node.js 20+
- pnpm 11+
- Rust stable
- macOS：Xcode Command Line Tools
- Windows：Microsoft C++ Build Tools 和 WebView2

```bash
pnpm install
pnpm tauri dev
```

仅预览前端：

```bash
pnpm dev
```

浏览器预览会使用内置示例数据；在 Tauri 窗口中会调用真实的本地扫描和更新命令。

生产构建：

```bash
pnpm build
pnpm tauri build
```

## CurseForge API Key 与数据源

一期数据源为 CurseForge。应用会调用其 REST API 的项目与文件接口，补全插件名称、简介、官方网站、最新版本与下载地址。

默认 `x-api-key` 从原生进程的 `CURSEFORGE_API_KEY` 读取：开发时可由启动环境提供，打包时会注入二进制。它不会显示或传递到 WebView。构建前可执行：

```bash
export CURSEFORGE_API_KEY='your-key'
pnpm tauri build
```

用户也可以在“设置 → 插件信息数据源”中填写个人 Key；个人 Key 优先于默认 Key，且仅在开启“记住个人 API Key”后以明文保存到本机。下载文件时不会向 CurseForge CDN 发送任何 API Key。桌面应用中的构建期 Key 不能视为不可提取的机密，建议使用可轮换、权限受限的 Key。

数据源选择器已为 WoWInterface 预留入口；一期仅启用 CurseForge，选择 WoWInterface 时会提示其仍在开发中。

## 本地目录与备份

默认扫描目录：

```text
World of Warcraft/
├── _retail_/Interface/AddOns
├── _classic_/Interface/AddOns
├── _classic_era_/Interface/AddOns
├── _anniversary_/Interface/AddOns
├── _classic_titan_/Interface/AddOns
├── _classic_anniversary_/Interface/AddOns  # 旧版兼容
├── _ptr_/Interface/AddOns
└── _beta_/Interface/AddOns
```

更新前的旧版本保存在当前 `AddOns/.wowbox-backups/` 下。备份不上传，也不会被插件扫描器识别。

在“设置 → 游戏目录”中可以为每个客户端版本单独选择路径。单独路径优先于统一根目录和自动检测，并仅保存在本机；未设置的版本仍会按统一根目录或默认目录自动发现。

## 项目结构

```text
src/
├── App.vue                 # 主界面和一期交互
├── data/mock.ts            # 浏览器预览数据
├── services/bridge.ts      # Tauri 命令桥接
└── types.ts                # 前后端共享的数据形状

src-tauri/src/
├── scanner.rs              # 客户端检测与 TOC 扫描
├── provider_config.rs      # 原生进程内的 CurseForge Key 解析
├── providers.rs            # CurseForge REST API 查询
├── updater.rs              # 下载、备份、安全解压、回滚
├── version.rs              # 非标准版本号自然排序
└── models.rs               # Tauri 命令数据模型
```

## 一期限制

- 只有 `.toc` 中带 `X-Curse-Project-ID` 的插件才能在一期自动关联更新源；其余插件会标记为“未关联”。
- CurseForge 下载是否可用取决于默认或个人 API Key 的权限，以及插件作者的分发设置。
- 当前不包含插件市场、账号同步、云端配置或 WeakAuras 管理。
