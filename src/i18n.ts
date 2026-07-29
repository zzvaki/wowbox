import type { AppLocale } from "@/types";

type MessageValues = Record<string, number | string>;

const messages = {
  "zh-CN": {
    addonManager: "插件管理器",
    gameVersions: "游戏版本",
    addonCount: "{count} 个插件",
    noGameClient: "未找到游戏客户端",
    manualSelect: "手动选择",
    settings: "设置",
    wowClient: "魔兽世界客户端",
    rescan: "重新扫描",
    rescanAddonFolder: "重新扫描插件目录",
    checkUpdates: "检查更新",
    updateAll: "全部更新",
    installedAddons: "已安装插件",
    scannedAt: "扫描于 {time}",
    updatesAvailable: "可用更新",
    checkedAt: "{time} 已检查",
    waitingForCheck: "等待检查",
    currentDirectory: "当前目录",
    chooseGameDirectory: "请先选择游戏目录",
    myAddons: "我的插件",
    manageAddons: "管理已安装内容和更新来源",
    searchAddons: "搜索插件、作者…",
    addon: "插件",
    source: "来源",
    localVersion: "本地版本",
    status: "状态",
    unknown: "未知",
    unknownAuthor: "未知作者",
    createdBy: "由 {author} 创建",
    update: "更新",
    details: "详情",
    noMatchingAddons: "没有符合筛选条件的插件",
    noAddonsFound: "这个版本还没有扫描到插件",
    localOnly: "所有操作均在本机完成",
    gameDirectory: "游戏目录",
    gameDirectoryHelp: "自动识别目录中的多个客户端版本",
    gameRoot: "World of Warcraft 根目录",
    blankAutoDetect: "留空则自动检测",
    select: "选择",
    auto: "自动",
    clientPaths: "按版本指定客户端目录",
    clientPathsHelp: "默认收起；展开后可覆盖自动检测路径。",
    autoDetectedPath: "使用自动检测路径",
    clear: "清除",
    language: "语言",
    languageHelp: "默认跟随本机语言，也可手动切换。",
    dataSources: "插件信息数据源",
    dataSourcesHelp: "一期使用 CurseForge REST API，后续可扩展其他来源。",
    personalApiKey: "个人 CurseForge API Key（可选）",
    defaultApiKey: "留空使用应用默认 x-api-key",
    showKey: "显示密钥",
    hideKey: "隐藏密钥",
    rememberKey: "记住个人 API Key",
    rememberKeyHelp: "仅在填写个人 Key 时以明文保存到当前设备；共享设备请关闭",
    checkOnLaunch: "启动时检查更新",
    checkOnLaunchHelp: "扫描完成后自动查询已关联的插件",
    privacy: "WowBox 不上传插件列表、游戏路径或任何账号信息。",
    cancel: "取消",
    saveSettings: "保存设置",
    addonDetails: "插件详情",
    noDescription: "暂无插件描述",
    author: "作者",
    latestVersion: "最新版本",
    notChecked: "尚未检查",
    interface: "Interface",
    folderCount: "目录数量",
    folders: "包含目录",
    notPerformed: "尚未执行",
    statusCurrent: "已是最新",
    statusUpdate: "可更新",
    statusUntracked: "未关联",
    statusChecking: "检查中",
    statusUpdating: "更新中",
    statusError: "检查失败",
    sourceWowInterface: "WoWInterface（一期未启用）",
    sourceLocal: "本地插件",
    filterAll: "全部插件 · {count}",
    filterUpdates: "可更新 · {count}",
    filterCurrent: "已是最新 · {count}",
    filterUntracked: "未关联 · {count}",
    sourceCurseForge: "CurseForge（已启用）",
    sourceWowInterfaceChoice: "WoWInterface（开发中）",
    gameRetail: "正式服",
    gameClassic: "经典进度服",
    gameClassicEra: "经典旧世",
    gameAnniversary: "周年纪念服",
    gameTitan: "泰坦重铸时光服",
    gameClassicPtr: "经典测试服",
    gamePtr: "正式服测试服",
    gameBeta: "Beta 客户端",
    savedSettings: "设置已保存",
    saveDirectoryError: "无法保存游戏目录授权",
    restoreDirectoryError: "无法还原游戏目录授权",
    gameDirectoryNotFound: "没有找到游戏目录",
    addonsDetected: "已识别 {count} 个插件",
    scanFailed: "扫描插件失败",
    wowInterfacePending: "WoWInterface 数据源正在开发中；一期请切换为 CurseForge 后检查更新。",
    updatesFound: "发现 {count} 个可用更新",
    allUpToDate: "所有插件均为最新",
    checkUpdatesFailed: "检查更新失败",
    noDownload: "更新源没有返回可下载文件",
    addonUpdated: "{title} 已更新",
    updateFailed: "更新失败",
    addonUpdateFailed: "{title} 更新失败",
    allUpdated: "{count} 个插件已全部更新",
    updatesPartial: "已更新 {succeeded}/{total} 个插件",
    clientNotInDirectory: "所选目录中没有找到 {label} 客户端",
    clientPathSaved: "{label} 路径已设置",
    clientPathDetectFailed: "无法识别 {label} 目录",
  },
  "zh-TW": {
    addonManager: "插件管理器", gameVersions: "遊戲版本", addonCount: "{count} 個插件", noGameClient: "找不到遊戲用戶端", manualSelect: "手動選擇", settings: "設定", wowClient: "魔獸世界用戶端", rescan: "重新掃描", rescanAddonFolder: "重新掃描插件目錄", checkUpdates: "檢查更新", updateAll: "全部更新", installedAddons: "已安裝插件", scannedAt: "掃描於 {time}", updatesAvailable: "可用更新", checkedAt: "已於 {time} 檢查", waitingForCheck: "等待檢查", currentDirectory: "目前目錄", chooseGameDirectory: "請先選擇遊戲目錄", myAddons: "我的插件", manageAddons: "管理已安裝內容與更新來源", searchAddons: "搜尋插件、作者…", addon: "插件", source: "來源", localVersion: "本機版本", status: "狀態", unknown: "未知", unknownAuthor: "未知作者", createdBy: "由 {author} 建立", update: "更新", details: "詳細資料", noMatchingAddons: "沒有符合篩選條件的插件", noAddonsFound: "這個版本尚未掃描到插件", localOnly: "所有操作均在本機完成", gameDirectory: "遊戲目錄", gameDirectoryHelp: "自動識別目錄中的多個用戶端版本", gameRoot: "World of Warcraft 根目錄", blankAutoDetect: "留空則自動偵測", select: "選擇", auto: "自動", clientPaths: "依版本指定用戶端目錄", clientPathsHelp: "預設收起；展開後可覆蓋自動偵測路徑。", autoDetectedPath: "使用自動偵測路徑", clear: "清除", language: "語言", languageHelp: "預設跟隨本機語言，也可手動切換。", dataSources: "插件資訊資料來源", dataSourcesHelp: "一期使用 CurseForge REST API，後續可擴充其他來源。", personalApiKey: "個人 CurseForge API Key（選填）", defaultApiKey: "留空使用應用程式預設 x-api-key", showKey: "顯示金鑰", hideKey: "隱藏金鑰", rememberKey: "記住個人 API Key", rememberKeyHelp: "僅在填寫個人 Key 時以明文儲存於本機；共享裝置請關閉", checkOnLaunch: "啟動時檢查更新", checkOnLaunchHelp: "掃描完成後自動查詢已關聯的插件", privacy: "WowBox 不會上傳插件清單、遊戲路徑或任何帳號資訊。", cancel: "取消", saveSettings: "儲存設定", addonDetails: "插件詳細資料", noDescription: "暫無插件說明", author: "作者", latestVersion: "最新版本", notChecked: "尚未檢查", interface: "Interface", folderCount: "目錄數量", folders: "包含目錄", notPerformed: "尚未執行", statusCurrent: "已是最新", statusUpdate: "可更新", statusUntracked: "未關聯", statusChecking: "檢查中", statusUpdating: "更新中", statusError: "檢查失敗", sourceWowInterface: "WoWInterface（一期未啟用）", sourceLocal: "本機插件", filterAll: "全部插件 · {count}", filterUpdates: "可更新 · {count}", filterCurrent: "已是最新 · {count}", filterUntracked: "未關聯 · {count}", sourceCurseForge: "CurseForge（已啟用）", sourceWowInterfaceChoice: "WoWInterface（開發中）", gameRetail: "正式服", gameClassic: "經典進度服", gameClassicEra: "經典舊世", gameAnniversary: "週年紀念服", gameTitan: "泰坦重鑄時光服", gameClassicPtr: "經典測試服", gamePtr: "正式服測試服", gameBeta: "Beta 用戶端", savedSettings: "設定已儲存", saveDirectoryError: "無法儲存遊戲目錄授權", restoreDirectoryError: "無法還原遊戲目錄授權", gameDirectoryNotFound: "找不到遊戲目錄", addonsDetected: "已識別 {count} 個插件", scanFailed: "掃描插件失敗", wowInterfacePending: "WoWInterface 資料來源正在開發中；一期請切換為 CurseForge 後檢查更新。", updatesFound: "發現 {count} 個可用更新", allUpToDate: "所有插件均為最新", checkUpdatesFailed: "檢查更新失敗", noDownload: "更新來源沒有回傳可下載檔案", addonUpdated: "{title} 已更新", updateFailed: "更新失敗", addonUpdateFailed: "{title} 更新失敗", allUpdated: "{count} 個插件已全部更新", updatesPartial: "已更新 {succeeded}/{total} 個插件", clientNotInDirectory: "所選目錄中找不到 {label} 用戶端", clientPathSaved: "{label} 路徑已設定", clientPathDetectFailed: "無法識別 {label} 目錄" 
  },
  "en-US": {
    addonManager: "Add-on Manager", gameVersions: "GAME VERSIONS", addonCount: "{count} add-ons", noGameClient: "No game client found", manualSelect: "Choose manually", settings: "Settings", wowClient: "World of Warcraft Client", rescan: "Rescan", rescanAddonFolder: "Rescan add-on folder", checkUpdates: "Check for updates", updateAll: "Update all", installedAddons: "Installed add-ons", scannedAt: "Scanned {time}", updatesAvailable: "Updates available", checkedAt: "Checked {time}", waitingForCheck: "Waiting to check", currentDirectory: "Current directory", chooseGameDirectory: "Choose a game directory first", myAddons: "My Add-ons", manageAddons: "Manage installed add-ons and update sources", searchAddons: "Search add-ons or authors…", addon: "Add-on", source: "Source", localVersion: "Local version", status: "Status", unknown: "Unknown", unknownAuthor: "Unknown author", createdBy: "Created by {author}", update: "Update", details: "Details", noMatchingAddons: "No add-ons match the current filters", noAddonsFound: "No add-ons were found for this version", localOnly: "All operations stay on this device", gameDirectory: "Game directory", gameDirectoryHelp: "Automatically detect multiple client versions in this directory", gameRoot: "World of Warcraft root directory", blankAutoDetect: "Leave empty to detect automatically", select: "Choose", auto: "Auto", clientPaths: "Set client directory by version", clientPathsHelp: "Collapsed by default; expand to override auto-detected paths.", autoDetectedPath: "Use auto-detected path", clear: "Clear", language: "Language", languageHelp: "Defaults to your system language; you can change it anytime.", dataSources: "Add-on information source", dataSourcesHelp: "Phase one uses the CurseForge REST API; more sources can be added later.", personalApiKey: "Personal CurseForge API Key (optional)", defaultApiKey: "Leave empty to use the app default x-api-key", showKey: "Show key", hideKey: "Hide key", rememberKey: "Remember personal API Key", rememberKeyHelp: "Stored in plain text on this device only; turn off on shared devices", checkOnLaunch: "Check for updates on launch", checkOnLaunchHelp: "Query linked add-ons automatically after scanning", privacy: "WowBox does not upload your add-on list, game paths, or account information.", cancel: "Cancel", saveSettings: "Save settings", addonDetails: "Add-on details", noDescription: "No add-on description", author: "Author", latestVersion: "Latest version", notChecked: "Not checked", interface: "Interface", folderCount: "Folders", folders: "Included folders", notPerformed: "Not performed", statusCurrent: "Up to date", statusUpdate: "Update available", statusUntracked: "Untracked", statusChecking: "Checking", statusUpdating: "Updating", statusError: "Check failed", sourceWowInterface: "WoWInterface (not enabled in phase one)", sourceLocal: "Local add-on", filterAll: "All add-ons · {count}", filterUpdates: "Updates · {count}", filterCurrent: "Up to date · {count}", filterUntracked: "Untracked · {count}", sourceCurseForge: "CurseForge (enabled)", sourceWowInterfaceChoice: "WoWInterface (in development)", gameRetail: "Retail", gameClassic: "Classic progression", gameClassicEra: "Classic Era", gameAnniversary: "Anniversary", gameTitan: "Titan Reforged", gameClassicPtr: "Classic PTR", gamePtr: "Retail PTR", gameBeta: "Beta client", savedSettings: "Settings saved", saveDirectoryError: "Could not save game directory access", restoreDirectoryError: "Could not restore game directory access", gameDirectoryNotFound: "No game directory found", addonsDetected: "Identified {count} add-ons", scanFailed: "Could not scan add-ons", wowInterfacePending: "The WoWInterface source is in development. Use CurseForge for phase one update checks.", updatesFound: "Found {count} available updates", allUpToDate: "All add-ons are up to date", checkUpdatesFailed: "Could not check for updates", noDownload: "The update source did not return a downloadable file", addonUpdated: "{title} updated", updateFailed: "Update failed", addonUpdateFailed: "Could not update {title}", allUpdated: "Updated all {count} add-ons", updatesPartial: "Updated {succeeded}/{total} add-ons", clientNotInDirectory: "No {label} client was found in the selected directory", clientPathSaved: "{label} path saved", clientPathDetectFailed: "Could not detect the {label} directory"
  },
  "ja-JP": {
    addonManager: "アドオンマネージャー", gameVersions: "ゲームバージョン", addonCount: "{count} 個のアドオン", noGameClient: "ゲームクライアントが見つかりません", manualSelect: "手動で選択", settings: "設定", wowClient: "World of Warcraft クライアント", rescan: "再スキャン", rescanAddonFolder: "アドオンフォルダーを再スキャン", checkUpdates: "更新を確認", updateAll: "すべて更新", installedAddons: "インストール済みアドオン", scannedAt: "スキャン: {time}", updatesAvailable: "利用可能な更新", checkedAt: "確認済み: {time}", waitingForCheck: "確認待ち", currentDirectory: "現在のディレクトリ", chooseGameDirectory: "先にゲームディレクトリを選択してください", myAddons: "マイアドオン", manageAddons: "インストール済みアドオンと更新元を管理", searchAddons: "アドオンまたは作者を検索…", addon: "アドオン", source: "ソース", localVersion: "ローカルバージョン", status: "状態", unknown: "不明", unknownAuthor: "不明な作者", createdBy: "作成者: {author}", update: "更新", details: "詳細", noMatchingAddons: "フィルターに一致するアドオンはありません", noAddonsFound: "このバージョンではアドオンが見つかりませんでした", localOnly: "すべての操作はこのデバイス内で完結します", gameDirectory: "ゲームディレクトリ", gameDirectoryHelp: "このディレクトリ内の複数のクライアントバージョンを自動検出します", gameRoot: "World of Warcraft ルートディレクトリ", blankAutoDetect: "空欄で自動検出", select: "選択", auto: "自動", clientPaths: "バージョンごとにクライアントディレクトリを指定", clientPathsHelp: "初期状態では折りたたまれています。展開すると自動検出パスを上書きできます。", autoDetectedPath: "自動検出パスを使用", clear: "クリア", language: "言語", languageHelp: "初期値はシステム言語です。いつでも変更できます。", dataSources: "アドオン情報のソース", dataSourcesHelp: "第1期は CurseForge REST API を使用します。後から他のソースを追加できます。", personalApiKey: "個人用 CurseForge API Key（任意）", defaultApiKey: "空欄でアプリ既定の x-api-key を使用", showKey: "キーを表示", hideKey: "キーを隠す", rememberKey: "個人用 API Key を記憶", rememberKeyHelp: "このデバイスに平文で保存されます。共有デバイスではオフにしてください", checkOnLaunch: "起動時に更新を確認", checkOnLaunchHelp: "スキャン後に紐付け済みアドオンを自動照会します", privacy: "WowBox はアドオン一覧、ゲームパス、アカウント情報を送信しません。", cancel: "キャンセル", saveSettings: "設定を保存", addonDetails: "アドオンの詳細", noDescription: "アドオンの説明はありません", author: "作者", latestVersion: "最新バージョン", notChecked: "未確認", interface: "Interface", folderCount: "フォルダー数", folders: "含まれるフォルダー", notPerformed: "未実行", statusCurrent: "最新です", statusUpdate: "更新可能", statusUntracked: "未関連", statusChecking: "確認中", statusUpdating: "更新中", statusError: "確認失敗", sourceWowInterface: "WoWInterface（第1期では未対応）", sourceLocal: "ローカルアドオン", filterAll: "すべて · {count}", filterUpdates: "更新可能 · {count}", filterCurrent: "最新 · {count}", filterUntracked: "未関連 · {count}", sourceCurseForge: "CurseForge（有効）", sourceWowInterfaceChoice: "WoWInterface（開発中）", gameRetail: "リテール", gameClassic: "クラシック進行", gameClassicEra: "クラシック Era", gameAnniversary: "アニバーサリー", gameTitan: "Titan Reforged", gameClassicPtr: "クラシック PTR", gamePtr: "リテール PTR", gameBeta: "ベータクライアント", savedSettings: "設定を保存しました", saveDirectoryError: "ゲームディレクトリのアクセスを保存できませんでした", restoreDirectoryError: "ゲームディレクトリのアクセスを復元できませんでした", gameDirectoryNotFound: "ゲームディレクトリが見つかりません", addonsDetected: "{count} 個のアドオンを識別しました", scanFailed: "アドオンをスキャンできませんでした", wowInterfacePending: "WoWInterface ソースは開発中です。第1期の更新確認には CurseForge を使用してください。", updatesFound: "{count} 件の更新が見つかりました", allUpToDate: "すべてのアドオンは最新です", checkUpdatesFailed: "更新を確認できませんでした", noDownload: "更新元からダウンロード可能なファイルが返されませんでした", addonUpdated: "{title} を更新しました", updateFailed: "更新に失敗しました", addonUpdateFailed: "{title} を更新できませんでした", allUpdated: "{count} 個のアドオンをすべて更新しました", updatesPartial: "{succeeded}/{total} 個のアドオンを更新しました", clientNotInDirectory: "選択したディレクトリに {label} クライアントがありません", clientPathSaved: "{label} のパスを保存しました", clientPathDetectFailed: "{label} ディレクトリを検出できませんでした"
  },
} as const;

export type TranslationKey = keyof (typeof messages)["zh-CN"];

export const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "繁體中文", value: "zh-TW" },
  { label: "English", value: "en-US" },
  { label: "日本語", value: "ja-JP" },
] satisfies Array<{ label: string; value: AppLocale }>;

export function isAppLocale(value: unknown): value is AppLocale {
  return value === "zh-CN" || value === "zh-TW" || value === "en-US" || value === "ja-JP";
}

export function detectLocale(languages = navigator.languages): AppLocale {
  const preferred = languages.find(Boolean)?.toLowerCase() ?? navigator.language.toLowerCase();
  if (preferred.startsWith("ja")) return "ja-JP";
  if (preferred.startsWith("zh")) {
    return /tw|hk|mo|hant/.test(preferred) ? "zh-TW" : "zh-CN";
  }
  return "en-US";
}

export function translate(
  locale: AppLocale,
  key: TranslationKey,
  values: MessageValues = {},
): string {
  return messages[locale][key].replace(/\{(\w+)\}/g, (_, name: string) =>
    String(values[name] ?? `{${name}}`),
  );
}
