<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  NButton,
  NCollapse,
  NCollapseItem,
  NConfigProvider,
  NEmpty,
  NFormItem,
  NInput,
  NMessageProvider,
  NModal,
  NProgress,
  NSelect,
  NSkeleton,
  NSwitch,
  NTag,
  NTooltip,
  createDiscreteApi,
  type GlobalThemeOverrides,
} from "naive-ui";
import {
  AlertCircle,
  Check,
  CheckCircle2,
  ChevronRight,
  CircleHelp,
  Download,
  Eye,
  EyeOff,
  FolderOpen,
  FolderSearch,
  HardDrive,
  Info,
  LoaderCircle,
  PackageCheck,
  Puzzle,
  RefreshCw,
  Search,
  Settings2,
  ShieldCheck,
  Sparkles,
} from "lucide-vue-next";
import {
  checkAddonUpdates,
  chooseGameRoot,
  detectInstallations,
  installAddonUpdate,
  scanAddons,
  syncAuthorizedGameRoots,
} from "@/services/bridge";
import type {
  AddonInfo,
  AddonSource,
  AddonStatus,
  AppSettings,
  GameFlavor,
  GameInstallation,
  PluginDataSource,
} from "@/types";

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#5f6ff4",
    primaryColorHover: "#5363e8",
    primaryColorPressed: "#4656d7",
    primaryColorSuppl: "#7684ff",
    borderRadius: "10px",
    fontFamily:
      '"Inter", "SF Pro Display", "PingFang SC", "Microsoft YaHei", sans-serif',
  },
  Button: {
    borderRadiusMedium: "9px",
    heightMedium: "38px",
  },
  Input: {
    borderRadius: "9px",
  },
};

const { message } = createDiscreteApi(["message"], {
  configProviderProps: {
    theme: undefined,
    themeOverrides,
  },
});

const defaultSettings: AppSettings = {
  gameRoot: "",
  clientPaths: {},
  pluginDataSource: "curseforge",
  curseforgeApiKey: "",
  rememberApiKey: false,
  checkOnLaunch: false,
};

function cloneSettings(value: AppSettings): AppSettings {
  return {
    ...value,
    clientPaths: { ...value.clientPaths },
  };
}

const clientPathOptions: Array<{
  flavor: GameFlavor;
  label: string;
  folder: string;
}> = [
  { flavor: "retail", label: "正式服", folder: "_retail_" },
  { flavor: "classic", label: "经典进度服", folder: "_classic_" },
  { flavor: "classic_era", label: "经典旧世", folder: "_classic_era_" },
  {
    flavor: "classic_anniversary",
    label: "周年纪念服",
    folder: "_anniversary_",
  },
  {
    flavor: "classic_titan",
    label: "泰坦重铸时光服",
    folder: "_classic_titan_",
  },
  { flavor: "classic_ptr", label: "经典测试服", folder: "_classic_ptr_" },
  { flavor: "ptr", label: "正式服测试服", folder: "_ptr_" },
  { flavor: "beta", label: "Beta 客户端", folder: "_beta_" },
];

const pluginDataSourceOptions = [
  {
    label: "CurseForge",
    value: "curseforge" satisfies PluginDataSource,
  },
  {
    label: "WoWInterface（即将支持）",
    value: "wowinterface",
    disabled: true,
  },
];

const installations = ref<GameInstallation[]>([]);
const activeInstallationId = ref("");
const addons = ref<AddonInfo[]>([]);
const searchTerm = ref("");
const statusFilter = ref<"all" | "update" | "current" | "untracked">("all");
const loadingInstallations = ref(true);
const scanning = ref(false);
const checking = ref(false);
const updatingAll = ref(false);
const settingsVisible = ref(false);
const expandedSettings = ref<string[]>([]);
const detailsVisible = ref(false);
const selectedAddon = ref<AddonInfo | null>(null);
const apiKeyVisible = ref(false);
const settings = ref<AppSettings>(cloneSettings(defaultSettings));
const settingsDraft = ref<AppSettings>(cloneSettings(defaultSettings));
const scanCompletedAt = ref<Date | null>(null);
const checkedAt = ref<Date | null>(null);

const activeInstallation = computed(() =>
  installations.value.find(
    (installation) => installation.id === activeInstallationId.value,
  ),
);

const updateCount = computed(
  () => addons.value.filter((addon) => addon.status === "update").length,
);

const currentCount = computed(
  () => addons.value.filter((addon) => addon.status === "current").length,
);

const visibleAddons = computed(() => {
  const query = searchTerm.value.trim().toLocaleLowerCase();
  return addons.value.filter((addon) => {
    const matchesStatus =
      statusFilter.value === "all" || addon.status === statusFilter.value;
    const matchesSearch =
      !query ||
      [addon.title, addon.author, addon.folderName, addon.notes].some((value) =>
        value.toLocaleLowerCase().includes(query),
      );
    return matchesStatus && matchesSearch;
  });
});

const filterOptions = computed(() => [
  { label: `全部插件 · ${addons.value.length}`, value: "all" },
  { label: `可更新 · ${updateCount.value}`, value: "update" },
  { label: `已是最新 · ${currentCount.value}`, value: "current" },
  {
    label: `未关联 · ${
      addons.value.filter((addon) => addon.status === "untracked").length
    }`,
    value: "untracked",
  },
]);

const progressPercentage = computed(() => {
  if (!addons.value.length) return 0;
  return Math.round((currentCount.value / addons.value.length) * 100);
});

function loadSettings() {
  try {
    const raw = localStorage.getItem("wowbox:settings");
    if (!raw) return;
    const stored = JSON.parse(raw) as Partial<AppSettings>;
    settings.value = {
      ...defaultSettings,
      ...stored,
      clientPaths: stored.clientPaths ?? {},
      pluginDataSource: stored.pluginDataSource ?? "curseforge",
      curseforgeApiKey: stored.rememberApiKey
        ? (stored.curseforgeApiKey ?? "")
        : "",
    };
    settingsDraft.value = cloneSettings(settings.value);
  } catch {
    localStorage.removeItem("wowbox:settings");
  }
}

function configuredGamePaths(value: AppSettings) {
  return [value.gameRoot, ...Object.values(value.clientPaths)].filter(
    (path): path is string => Boolean(path?.trim()),
  );
}

function openSettings() {
  settingsDraft.value = cloneSettings(settings.value);
  expandedSettings.value = [];
  settingsVisible.value = true;
}

async function saveSettings() {
  const nextSettings = cloneSettings(settingsDraft.value);
  try {
    await syncAuthorizedGameRoots(configuredGamePaths(nextSettings));
  } catch (error) {
    message.error(errorMessage(error, "无法保存游戏目录授权"));
    return;
  }
  const stored = {
    ...nextSettings,
    curseforgeApiKey: nextSettings.rememberApiKey
      ? nextSettings.curseforgeApiKey
      : "",
  };
  localStorage.setItem("wowbox:settings", JSON.stringify(stored));
  settings.value = nextSettings;
  settingsVisible.value = false;
  message.success("设置已保存");
  await refreshInstallations();
}

async function cancelSettings() {
  try {
    await syncAuthorizedGameRoots(configuredGamePaths(settings.value));
  } catch (error) {
    message.error(errorMessage(error, "无法还原游戏目录授权"));
    return;
  }
  settingsDraft.value = cloneSettings(settings.value);
  settingsVisible.value = false;
}

async function refreshInstallations() {
  loadingInstallations.value = true;
  try {
    const configuredPaths = configuredGamePaths(settings.value);
    const detectionPaths = Array.from(
      new Set(
        settings.value.gameRoot
          ? [settings.value.gameRoot, ...configuredPaths]
          : [undefined, ...configuredPaths],
      ),
    );
    const detectionResults = await Promise.allSettled(
      detectionPaths.map((path) => detectInstallations(path)),
    );
    const detected = detectionResults.flatMap((result) =>
      result.status === "fulfilled" ? result.value : [],
    );
    if (!detected.length) {
      const firstError = detectionResults.find(
        (result): result is PromiseRejectedResult => result.status === "rejected",
      );
      throw firstError?.reason ?? new Error("没有找到可用客户端");
    }

    const installationsByFlavor = new Map<GameFlavor, GameInstallation>();
    for (const installation of detected) {
      if (!installationsByFlavor.has(installation.flavor)) {
        installationsByFlavor.set(installation.flavor, installation);
      }
    }
    for (const [flavor, path] of Object.entries(settings.value.clientPaths)) {
      const configuredInstallation = detected.find(
        (installation) =>
          installation.flavor === flavor && installation.path === path,
      );
      if (configuredInstallation) {
        installationsByFlavor.set(
          flavor as GameFlavor,
          configuredInstallation,
        );
      }
    }
    installations.value = Array.from(installationsByFlavor.values());
    const stillExists = installations.value.some(
      (installation) => installation.id === activeInstallationId.value,
    );
    if (!stillExists) {
      activeInstallationId.value = installations.value[0]?.id ?? "";
    }
    if (activeInstallation.value) {
      await runScan(false);
    } else {
      addons.value = [];
    }
  } catch (error) {
    message.error(errorMessage(error, "没有找到游戏目录"));
  } finally {
    loadingInstallations.value = false;
  }
}

async function selectInstallation(id: string) {
  if (id === activeInstallationId.value) return;
  activeInstallationId.value = id;
  searchTerm.value = "";
  statusFilter.value = "all";
  await runScan();
}

async function runScan(showNotice = true) {
  if (!activeInstallation.value || scanning.value) return;
  scanning.value = true;
  try {
    addons.value = await scanAddons(
      activeInstallation.value.addonsPath,
      activeInstallation.value.flavor,
    );
    scanCompletedAt.value = new Date();
    if (showNotice) {
      message.success(`已识别 ${addons.value.length} 个插件`);
    }
  } catch (error) {
    message.error(errorMessage(error, "扫描插件失败"));
  } finally {
    scanning.value = false;
  }
}

async function runUpdateCheck() {
  if (!activeInstallation.value || !addons.value.length || checking.value) return;
  checking.value = true;
  addons.value = addons.value.map((addon) => ({
    ...addon,
    status: addon.source === "unknown" ? "untracked" : "checking",
  }));
  try {
    const results = await checkAddonUpdates(
      addons.value,
      activeInstallation.value.flavor,
      settings.value.pluginDataSource,
      settings.value.curseforgeApiKey,
    );
    const resultMap = new Map(results.map((result) => [result.addonId, result]));
    addons.value = addons.value.map((addon) => {
      const result = resultMap.get(addon.id);
      if (!result) return { ...addon, status: "error" };
      return {
        ...addon,
        status: result.status,
        title: result.title || addon.title,
        notes: result.summary || addon.notes,
        latestVersion: result.latestVersion,
        latestFileId: result.latestFileId,
        latestDownloadUrl: result.downloadUrl,
        websiteUrl: result.websiteUrl,
        error: result.error,
      };
    });
    checkedAt.value = new Date();
    const available = results.filter((result) => result.status === "update").length;
    message.success(available ? `发现 ${available} 个可用更新` : "所有插件均为最新");
  } catch (error) {
    addons.value = addons.value.map((addon) => ({
      ...addon,
      status: addon.source === "unknown" ? "untracked" : "error",
    }));
    message.error(errorMessage(error, "检查更新失败"));
  } finally {
    checking.value = false;
  }
}

async function updateOne(addon: AddonInfo, quiet = false) {
  if (!addon.latestDownloadUrl) {
    message.warning("更新源没有返回可下载文件");
    return false;
  }
  const previousStatus = addon.status;
  addon.status = "updating";
  try {
    const result = await installAddonUpdate({
      addon,
      downloadUrl: addon.latestDownloadUrl,
      apiKey:
        addon.source === "curseforge"
          ? settings.value.curseforgeApiKey
          : undefined,
    });
    addon.version = result.version;
    addon.status = "current";
    if (!quiet) message.success(`${addon.title} 已更新`);
    return true;
  } catch (error) {
    addon.status = previousStatus;
    addon.error = errorMessage(error, "更新失败");
    if (!quiet) message.error(`${addon.title} 更新失败`);
    return false;
  }
}

async function updateAll() {
  const pending = addons.value.filter((addon) => addon.status === "update");
  if (!pending.length || updatingAll.value) return;
  updatingAll.value = true;
  let succeeded = 0;
  for (const addon of pending) {
    if (await updateOne(addon, true)) succeeded += 1;
  }
  updatingAll.value = false;
  if (succeeded === pending.length) {
    message.success(`${succeeded} 个插件已全部更新`);
  } else {
    message.warning(`已更新 ${succeeded}/${pending.length} 个插件`);
  }
}

async function selectGameRoot() {
  const path = await chooseGameRoot();
  if (path) settingsDraft.value.gameRoot = path;
}

async function selectClientPath(flavor: GameFlavor, label: string) {
  const path = await chooseGameRoot();
  if (!path) return;
  try {
    const detected = await detectInstallations(path);
    const installation = detected.find((item) => item.flavor === flavor);
    if (!installation) {
      message.warning(`所选目录中没有找到${label}客户端`);
      return;
    }
    settingsDraft.value.clientPaths[flavor] = installation.path;
    message.success(`${label}路径已设置`);
  } catch (error) {
    message.error(errorMessage(error, `无法识别${label}目录`));
  }
}

function clearClientPath(flavor: GameFlavor) {
  delete settingsDraft.value.clientPaths[flavor];
}

function showDetails(addon: AddonInfo) {
  selectedAddon.value = addon;
  detailsVisible.value = true;
}

function statusLabel(status: AddonStatus) {
  const labels: Record<AddonStatus, string> = {
    current: "已是最新",
    update: "可更新",
    untracked: "未关联",
    checking: "检查中",
    updating: "更新中",
    error: "检查失败",
  };
  return labels[status];
}

function sourceLabel(source: AddonSource) {
  return {
    curseforge: "CurseForge",
    wowinterface: "WoWInterface（一期未启用）",
    unknown: "本地插件",
  }[source];
}

function sourceMark(source: AddonSource) {
  return {
    curseforge: "CF",
    wowinterface: "WI",
    unknown: "L",
  }[source];
}

function formatRelativeTime(date: Date | null) {
  if (!date) return "尚未执行";
  return date.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function errorMessage(error: unknown, fallback: string) {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return fallback;
}

onMounted(async () => {
  loadSettings();
  await refreshInstallations();
  if (settings.value.checkOnLaunch && addons.value.length) {
    await runUpdateCheck();
  }
});
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <n-message-provider>
      <div class="app-shell">
        <aside class="sidebar">
          <div class="brand">
            <div class="brand-mark">
              <span>W</span>
              <i />
            </div>
            <div>
              <div class="brand-name">WowBox</div>
              <div class="brand-caption">插件管理器</div>
            </div>
          </div>

          <div class="sidebar-section-label">游戏版本</div>
          <div v-if="loadingInstallations" class="sidebar-skeleton">
            <n-skeleton v-for="item in 3" :key="item" height="56px" round />
          </div>
          <nav v-else class="installation-list">
            <button
              v-for="installation in installations"
              :key="installation.id"
              class="installation-item"
              :class="{ active: installation.id === activeInstallationId }"
              type="button"
              @click="selectInstallation(installation.id)"
            >
              <span class="game-icon">
                <ShieldCheck :size="18" :stroke-width="1.9" />
              </span>
              <span class="game-meta">
                <strong>{{ installation.label }}</strong>
                <small>{{ installation.addonCount }} 个插件</small>
              </span>
              <ChevronRight class="nav-chevron" :size="16" />
            </button>
          </nav>

          <div v-if="!loadingInstallations && !installations.length" class="no-game">
            <FolderSearch :size="22" />
            <span>未找到游戏客户端</span>
            <button type="button" @click="openSettings">
              手动选择
            </button>
          </div>

          <div class="sidebar-spacer" />

          <div class="health-card">
            <div class="health-top">
              <span class="health-icon"><PackageCheck :size="18" /></span>
              <span>插件健康度</span>
              <strong>{{ progressPercentage }}%</strong>
            </div>
            <n-progress
              type="line"
              :percentage="progressPercentage"
              :show-indicator="false"
              :height="5"
              color="#6e7bf7"
              rail-color="#e8eaf6"
              border-radius="4px"
            />
            <p>{{ updateCount ? `${updateCount} 个插件可以更新` : "一切都井井有条" }}</p>
          </div>

          <button class="sidebar-settings" type="button" @click="openSettings">
            <Settings2 :size="18" />
            <span>设置</span>
          </button>
        </aside>

        <main class="main">
          <header class="topbar">
            <div class="topbar-title">
              <div class="eyebrow">
                <span class="online-dot" />
                国服客户端
              </div>
              <h1>{{ activeInstallation?.label ?? "插件管理" }}</h1>
            </div>
            <div class="topbar-actions">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button
                    quaternary
                    circle
                    :loading="scanning"
                    :disabled="!activeInstallation"
                    aria-label="重新扫描"
                    @click="runScan()"
                  >
                    <template #icon><RefreshCw :size="18" /></template>
                  </n-button>
                </template>
                重新扫描插件目录
              </n-tooltip>
              <n-button
                secondary
                :loading="checking"
                :disabled="!addons.length"
                @click="runUpdateCheck"
              >
                <template #icon><Sparkles :size="17" /></template>
                检查更新
              </n-button>
              <n-button
                type="primary"
                :loading="updatingAll"
                :disabled="!updateCount"
                @click="updateAll"
              >
                <template #icon><Download :size="17" /></template>
                全部更新
                <span v-if="updateCount" class="button-count">{{ updateCount }}</span>
              </n-button>
            </div>
          </header>

          <div class="content">
            <section class="summary-grid">
              <article class="summary-card primary-card">
                <div class="summary-icon purple">
                  <Puzzle :size="21" />
                </div>
                <div>
                  <span>已安装插件</span>
                  <strong>{{ addons.length }}</strong>
                </div>
                <small>扫描于 {{ formatRelativeTime(scanCompletedAt) }}</small>
              </article>
              <article class="summary-card">
                <div class="summary-icon coral">
                  <Download :size="21" />
                </div>
                <div>
                  <span>可用更新</span>
                  <strong>{{ updateCount }}</strong>
                </div>
                <small>{{ checkedAt ? `${formatRelativeTime(checkedAt)} 已检查` : "等待检查" }}</small>
              </article>
              <article class="summary-card">
                <div class="summary-icon mint">
                  <HardDrive :size="21" />
                </div>
                <div>
                  <span>当前目录</span>
                  <strong class="folder-value">{{ activeInstallation?.productFolder ?? "—" }}</strong>
                </div>
                <n-tooltip v-if="activeInstallation" trigger="hover">
                  <template #trigger>
                    <small class="path-text">{{ activeInstallation.addonsPath }}</small>
                  </template>
                  {{ activeInstallation.addonsPath }}
                </n-tooltip>
                <small v-else>请先选择游戏目录</small>
              </article>
            </section>

            <section class="addons-panel">
              <div class="panel-toolbar">
                <div>
                  <h2>我的插件</h2>
                  <p>管理已安装内容和更新来源</p>
                </div>
                <div class="toolbar-controls">
                  <n-input
                    v-model:value="searchTerm"
                    clearable
                    placeholder="搜索插件、作者…"
                    class="search-input"
                  >
                    <template #prefix><Search :size="17" /></template>
                  </n-input>
                  <n-select
                    v-model:value="statusFilter"
                    :options="filterOptions"
                    class="filter-select"
                    :consistent-menu-width="false"
                  />
                </div>
              </div>

              <div class="table-header">
                <span>插件</span>
                <span>来源</span>
                <span>本地版本</span>
                <span>状态</span>
                <span />
              </div>

              <div v-if="scanning" class="table-loading">
                <div v-for="item in 5" :key="item" class="skeleton-row">
                  <n-skeleton height="42px" width="42px" round />
                  <div>
                    <n-skeleton text width="42%" />
                    <n-skeleton text width="70%" />
                  </div>
                  <n-skeleton text width="66px" />
                  <n-skeleton text width="72px" />
                  <n-skeleton text width="78px" />
                </div>
              </div>

              <div v-else-if="visibleAddons.length" class="addon-list">
                <article
                  v-for="addon in visibleAddons"
                  :key="addon.id"
                  class="addon-row"
                  :class="{ 'has-update': addon.status === 'update' }"
                  @dblclick="showDetails(addon)"
                >
                  <div class="addon-main">
                    <div class="addon-avatar" :data-source="addon.source">
                      {{ addon.title.slice(0, 1).toUpperCase() }}
                    </div>
                    <div class="addon-copy">
                      <div class="addon-title-line">
                        <strong>{{ addon.title }}</strong>
                        <n-tag
                          v-if="addon.interfaceVersion && addon.interfaceVersion !== '未知'"
                          size="tiny"
                          :bordered="false"
                          class="interface-tag"
                        >
                          {{ addon.interfaceVersion }}
                        </n-tag>
                      </div>
                      <p>{{ addon.notes || `由 ${addon.author || "未知作者"} 创建` }}</p>
                    </div>
                  </div>

                  <div class="source-cell">
                    <span class="source-mark" :data-source="addon.source">
                      {{ sourceMark(addon.source) }}
                    </span>
                    <span>{{ sourceLabel(addon.source) }}</span>
                  </div>

                  <div class="version-cell">
                    <span>{{ addon.version || "未知" }}</span>
                    <small v-if="addon.status === 'update'">
                      → {{ addon.latestVersion }}
                    </small>
                  </div>

                  <div class="status-cell" :data-status="addon.status">
                    <LoaderCircle
                      v-if="addon.status === 'checking' || addon.status === 'updating'"
                      class="spin"
                      :size="15"
                    />
                    <CheckCircle2 v-else-if="addon.status === 'current'" :size="15" />
                    <Download v-else-if="addon.status === 'update'" :size="15" />
                    <CircleHelp v-else-if="addon.status === 'untracked'" :size="15" />
                    <AlertCircle v-else :size="15" />
                    <span>{{ statusLabel(addon.status) }}</span>
                  </div>

                  <div class="row-actions">
                    <n-button
                      v-if="addon.status === 'update'"
                      type="primary"
                      size="small"
                      @click="updateOne(addon)"
                    >
                      更新
                    </n-button>
                    <n-button
                      v-else
                      quaternary
                      size="small"
                      @click="showDetails(addon)"
                    >
                      详情
                    </n-button>
                  </div>
                </article>
              </div>

              <n-empty
                v-else
                class="empty-state"
                :description="
                  addons.length ? '没有符合筛选条件的插件' : '这个版本还没有扫描到插件'
                "
              >
                <template #icon><Puzzle :size="42" :stroke-width="1.3" /></template>
                <template #extra>
                  <n-button v-if="activeInstallation" secondary @click="runScan()">
                    重新扫描
                  </n-button>
                  <n-button v-else type="primary" @click="openSettings">
                    选择游戏目录
                  </n-button>
                </template>
              </n-empty>
            </section>

            <footer class="content-footer">
              <span><ShieldCheck :size="14" /> 所有操作均在本机完成</span>
              <span>WowBox 0.1.0</span>
            </footer>
          </div>
        </main>

        <n-modal
          v-model:show="settingsVisible"
          preset="card"
          title="设置"
          class="settings-modal"
          :bordered="false"
          :mask-closable="false"
          :close-on-esc="false"
        >
          <div class="settings-section">
            <div class="settings-section-title">
              <FolderOpen :size="18" />
              <div>
                <strong>游戏目录</strong>
                <span>自动识别目录中的多个客户端版本</span>
              </div>
            </div>
            <n-form-item label="World of Warcraft 根目录">
              <div class="path-picker">
                <n-input
                  v-model:value="settingsDraft.gameRoot"
                  readonly
                  placeholder="留空则自动检测"
                />
                <n-button secondary @click="selectGameRoot">选择</n-button>
                <n-button
                  v-if="settingsDraft.gameRoot"
                  quaternary
                  @click="settingsDraft.gameRoot = ''"
                >
                  自动
                </n-button>
              </div>
            </n-form-item>
            <n-collapse
              v-model:expanded-names="expandedSettings"
              class="client-path-collapse"
            >
              <n-collapse-item name="client-paths">
                <template #header>
                  <div class="client-paths-heading">
                    <strong>按版本指定客户端目录</strong>
                    <span>默认收起；展开后可覆盖自动检测路径。</span>
                  </div>
                </template>
                <div class="client-path-list">
                  <div
                    v-for="option in clientPathOptions"
                    :key="option.flavor"
                    class="client-path-row"
                  >
                    <div class="client-path-label">
                      <strong>{{ option.label }}</strong>
                      <span>{{ option.folder }}</span>
                    </div>
                    <div class="client-path-picker">
                      <n-input
                        :value="settingsDraft.clientPaths[option.flavor]"
                        readonly
                        placeholder="使用自动检测路径"
                      />
                      <n-button
                        secondary
                        size="small"
                        @click="selectClientPath(option.flavor, option.label)"
                      >
                        选择
                      </n-button>
                      <n-button
                        v-if="settingsDraft.clientPaths[option.flavor]"
                        quaternary
                        size="small"
                        @click="clearClientPath(option.flavor)"
                      >
                        清除
                      </n-button>
                    </div>
                  </div>
                </div>
              </n-collapse-item>
            </n-collapse>
          </div>

          <div class="settings-divider" />

          <div class="settings-section">
            <div class="settings-section-title">
              <ShieldCheck :size="18" />
              <div>
                <strong>插件信息数据源</strong>
                <span>一期使用 CurseForge REST API，后续可扩展其他来源。</span>
              </div>
            </div>
            <n-form-item label="数据源">
              <n-select
                v-model:value="settingsDraft.pluginDataSource"
                :options="pluginDataSourceOptions"
              />
            </n-form-item>
            <n-form-item label="个人 CurseForge API Key（可选）">
              <n-input
                v-model:value="settingsDraft.curseforgeApiKey"
                :type="apiKeyVisible ? 'text' : 'password'"
                placeholder="留空使用应用默认 x-api-key"
              >
                <template #suffix>
                  <button
                    class="input-icon-button"
                    type="button"
                    :aria-label="apiKeyVisible ? '隐藏密钥' : '显示密钥'"
                    @click="apiKeyVisible = !apiKeyVisible"
                  >
                    <EyeOff v-if="apiKeyVisible" :size="16" />
                    <Eye v-else :size="16" />
                  </button>
                </template>
              </n-input>
            </n-form-item>
            <div class="setting-toggle">
              <div>
                <strong>记住个人 API Key</strong>
                <span>仅在填写个人 Key 时保存到当前设备</span>
              </div>
              <n-switch v-model:value="settingsDraft.rememberApiKey" />
            </div>
            <div class="setting-toggle">
              <div>
                <strong>启动时检查更新</strong>
                <span>扫描完成后自动查询已关联的插件</span>
              </div>
              <n-switch v-model:value="settingsDraft.checkOnLaunch" />
            </div>
          </div>

          <div class="privacy-note">
            <Info :size="16" />
            <span>WowBox 不上传插件列表、游戏路径或任何账号信息。</span>
          </div>

          <template #footer>
            <div class="modal-footer">
              <n-button @click="cancelSettings">取消</n-button>
              <n-button type="primary" @click="saveSettings">
                <template #icon><Check :size="16" /></template>
                保存设置
              </n-button>
            </div>
          </template>
        </n-modal>

        <n-modal
          v-model:show="detailsVisible"
          preset="card"
          class="details-modal"
          :bordered="false"
          title="插件详情"
        >
          <div v-if="selectedAddon" class="details-content">
            <div class="details-hero">
              <div class="addon-avatar large" :data-source="selectedAddon.source">
                {{ selectedAddon.title.slice(0, 1).toUpperCase() }}
              </div>
              <div>
                <h3>{{ selectedAddon.title }}</h3>
                <p>{{ selectedAddon.notes || "暂无插件描述" }}</p>
              </div>
            </div>
            <dl class="details-grid">
              <div><dt>作者</dt><dd>{{ selectedAddon.author || "未知" }}</dd></div>
              <div><dt>来源</dt><dd>{{ sourceLabel(selectedAddon.source) }}</dd></div>
              <div><dt>本地版本</dt><dd>{{ selectedAddon.version || "未知" }}</dd></div>
              <div><dt>最新版本</dt><dd>{{ selectedAddon.latestVersion || "尚未检查" }}</dd></div>
              <div><dt>Interface</dt><dd>{{ selectedAddon.interfaceVersion || "未知" }}</dd></div>
              <div><dt>目录数量</dt><dd>{{ selectedAddon.folders.length }}</dd></div>
            </dl>
            <div class="folder-list">
              <span>包含目录</span>
              <code v-for="folder in selectedAddon.folders" :key="folder">{{ folder }}</code>
            </div>
            <div v-if="selectedAddon.error" class="error-note">
              <AlertCircle :size="16" />
              {{ selectedAddon.error }}
            </div>
          </div>
        </n-modal>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>
