<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  dateEnUS,
  dateJaJP,
  dateZhCN,
  dateZhTW,
  enUS,
  jaJP,
  NButton,
  NCollapse,
  NCollapseItem,
  NConfigProvider,
  NEmpty,
  NFormItem,
  NInput,
  NMessageProvider,
  NModal,
  NSelect,
  NSkeleton,
  NSwitch,
  NTag,
  NTooltip,
  createDiscreteApi,
  type GlobalThemeOverrides,
  zhCN,
  zhTW,
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
  fetchAddonDetails,
  installAddonUpdate,
  scanAddons,
  syncAuthorizedGameRoots,
} from "@/services/bridge";
import {
  detectLocale,
  isAppLocale,
  languageOptions,
  translate,
  type TranslationKey,
} from "@/i18n";
import type {
  AddonInfo,
  AddonSource,
  AddonStatus,
  AppLocale,
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

const initialLocale = detectLocale();

const defaultSettings: AppSettings = {
  language: initialLocale,
  gameRoot: "",
  clientPaths: {},
  pluginDataSource: "curseforge",
  curseforgeApiKey: "",
  rememberApiKey: false,
  checkOnLaunch: false,
};

const locale = ref<AppLocale>(initialLocale);
const t = (key: TranslationKey, values?: Record<string, number | string>) =>
  translate(locale.value, key, values);

const naiveLocales = {
  "zh-CN": { locale: zhCN, dateLocale: dateZhCN },
  "zh-TW": { locale: zhTW, dateLocale: dateZhTW },
  "en-US": { locale: enUS, dateLocale: dateEnUS },
  "ja-JP": { locale: jaJP, dateLocale: dateJaJP },
};
const naiveLocale = computed(() => naiveLocales[locale.value].locale);
const naiveDateLocale = computed(() => naiveLocales[locale.value].dateLocale);

function cloneSettings(value: AppSettings): AppSettings {
  return {
    ...value,
    clientPaths: { ...value.clientPaths },
  };
}

const gameFlavorKeys: Record<GameFlavor, TranslationKey> = {
  retail: "gameRetail",
  classic: "gameClassic",
  classic_era: "gameClassicEra",
  classic_anniversary: "gameAnniversary",
  classic_titan: "gameTitan",
  classic_ptr: "gameClassicPtr",
  ptr: "gamePtr",
  beta: "gameBeta",
};

function gameFlavorLabel(flavor: GameFlavor) {
  return t(gameFlavorKeys[flavor]);
}

const clientPathOptions = computed(() => [
  { flavor: "retail" as const, label: gameFlavorLabel("retail"), folder: "_retail_" },
  { flavor: "classic" as const, label: gameFlavorLabel("classic"), folder: "_classic_" },
  { flavor: "classic_era" as const, label: gameFlavorLabel("classic_era"), folder: "_classic_era_" },
  { flavor: "classic_anniversary" as const, label: gameFlavorLabel("classic_anniversary"), folder: "_anniversary_" },
  { flavor: "classic_titan" as const, label: gameFlavorLabel("classic_titan"), folder: "_classic_titan_" },
  { flavor: "classic_ptr" as const, label: gameFlavorLabel("classic_ptr"), folder: "_classic_ptr_" },
  { flavor: "ptr" as const, label: gameFlavorLabel("ptr"), folder: "_ptr_" },
  { flavor: "beta" as const, label: gameFlavorLabel("beta"), folder: "_beta_" },
]);

const pluginDataSourceOptions = computed(() => [
  { label: t("sourceCurseForge"), value: "curseforge" satisfies PluginDataSource },
  { label: t("sourceWowInterfaceChoice"), value: "wowinterface" satisfies PluginDataSource },
]);

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
const detailsLoading = ref(false);
const detailsError = ref("");
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
  { label: t("filterAll", { count: addons.value.length }), value: "all" },
  { label: t("filterUpdates", { count: updateCount.value }), value: "update" },
  { label: t("filterCurrent", { count: currentCount.value }), value: "current" },
  {
    label: t("filterUntracked", {
      count: addons.value.filter((addon) => addon.status === "untracked").length,
    }),
    value: "untracked",
  },
]);

function loadSettings() {
  try {
    const raw = localStorage.getItem("wowbox:settings");
    if (!raw) return;
    const stored = JSON.parse(raw) as Partial<AppSettings>;
    settings.value = {
      ...defaultSettings,
      ...stored,
      language: isAppLocale(stored.language)
        ? stored.language
        : defaultSettings.language,
      clientPaths: stored.clientPaths ?? {},
      pluginDataSource: stored.pluginDataSource ?? "curseforge",
      curseforgeApiKey: stored.rememberApiKey
        ? (stored.curseforgeApiKey ?? "")
        : "",
    };
    locale.value = settings.value.language;
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
  apiKeyVisible.value = false;
  settingsVisible.value = true;
}

function persistSettings(value: AppSettings) {
  const stored = {
    ...value,
    curseforgeApiKey: value.rememberApiKey ? value.curseforgeApiKey : "",
  };
  localStorage.setItem("wowbox:settings", JSON.stringify(stored));
  settings.value = value;
  locale.value = value.language;
}

async function saveSettings() {
  const nextSettings = cloneSettings(settingsDraft.value);
  try {
    await syncAuthorizedGameRoots(configuredGamePaths(nextSettings));
  } catch (error) {
    if (nextSettings.language !== settings.value.language) {
      persistSettings({ ...settings.value, language: nextSettings.language });
    }
    message.error(errorMessage(error, t("saveDirectoryError")));
    return;
  }
  persistSettings(nextSettings);
  settingsVisible.value = false;
  message.success(t("savedSettings"));
  await refreshInstallations();
}

async function cancelSettings() {
  try {
    await syncAuthorizedGameRoots(configuredGamePaths(settings.value));
  } catch (error) {
    message.error(errorMessage(error, t("restoreDirectoryError")));
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
      throw firstError?.reason ?? new Error(t("gameDirectoryNotFound"));
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
    message.error(errorMessage(error, t("gameDirectoryNotFound")));
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
      locale.value,
    );
    scanCompletedAt.value = new Date();
    if (showNotice) {
      message.success(t("addonsDetected", { count: addons.value.length }));
    }
  } catch (error) {
    message.error(errorMessage(error, t("scanFailed")));
  } finally {
    scanning.value = false;
  }
}

async function runUpdateCheck() {
  if (!activeInstallation.value || !addons.value.length || checking.value) return;
  if (settings.value.pluginDataSource !== "curseforge") {
    message.info(t("wowInterfacePending"));
    return;
  }
  checking.value = true;
  addons.value = addons.value.map((addon) => ({
    ...addon,
    status: addon.source === "wowinterface" ? "untracked" : "checking",
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
        author: result.author || addon.author,
        notes: result.summary || addon.notes,
        source: result.sourceId ? ("curseforge" as const) : addon.source,
        sourceId: result.sourceId || addon.sourceId,
        latestVersion: result.latestVersion,
        latestFileId: result.latestFileId,
        latestDownloadUrl: result.downloadUrl,
        websiteUrl: result.websiteUrl,
        error: result.error,
      };
    });
    checkedAt.value = new Date();
    const available = results.filter((result) => result.status === "update").length;
    message.success(
      available ? t("updatesFound", { count: available }) : t("allUpToDate"),
    );
  } catch (error) {
    addons.value = addons.value.map((addon) => ({
      ...addon,
      status: addon.source === "wowinterface" ? "untracked" : "error",
    }));
    message.error(errorMessage(error, t("checkUpdatesFailed")));
  } finally {
    checking.value = false;
  }
}

async function updateOne(addon: AddonInfo, quiet = false) {
  if (!addon.latestDownloadUrl) {
    message.warning(t("noDownload"));
    return false;
  }
  const previousStatus = addon.status;
  addon.status = "updating";
  try {
    const result = await installAddonUpdate({
      addon,
      downloadUrl: addon.latestDownloadUrl,
    });
    addon.version = result.version;
    addon.status = "current";
    if (!quiet) message.success(t("addonUpdated", { title: addon.title }));
    return true;
  } catch (error) {
    addon.status = previousStatus;
    addon.error = errorMessage(error, t("updateFailed"));
    if (!quiet) message.error(t("addonUpdateFailed", { title: addon.title }));
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
    message.success(t("allUpdated", { count: succeeded }));
  } else {
    message.warning(
      t("updatesPartial", { succeeded, total: pending.length }),
    );
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
      message.warning(t("clientNotInDirectory", { label }));
      return;
    }
    settingsDraft.value.clientPaths[flavor] = installation.path;
    message.success(t("clientPathSaved", { label }));
  } catch (error) {
    message.error(errorMessage(error, t("clientPathDetectFailed", { label })));
  }
}

function clearClientPath(flavor: GameFlavor) {
  delete settingsDraft.value.clientPaths[flavor];
}

async function showDetails(addon: AddonInfo) {
  selectedAddon.value = addon;
  detailsLoading.value = false;
  detailsError.value = "";
  detailsVisible.value = true;
  if (
    addon.remoteDetails ||
    addon.source === "wowinterface" ||
    settings.value.pluginDataSource !== "curseforge" ||
    !activeInstallation.value
  ) {
    return;
  }

  const requestedAddonId = addon.id;
  detailsLoading.value = true;
  try {
    const details = await fetchAddonDetails(
      addon,
      activeInstallation.value.flavor,
      settings.value.curseforgeApiKey,
    );
    Object.assign(addon, {
      title: details.name || addon.title,
      notes: details.summary || addon.notes,
      author:
        details.authors.map((author) => author.name).filter(Boolean).join(", ") ||
        addon.author,
      source: "curseforge" as const,
      sourceId: details.projectId,
      websiteUrl: details.websiteUrl || addon.websiteUrl,
      remoteDetails: details,
    });
    if (selectedAddon.value?.id === requestedAddonId) {
      selectedAddon.value = addon;
    }
  } catch (error) {
    if (selectedAddon.value?.id === requestedAddonId) {
      detailsError.value = errorMessage(error, t("detailsLoadFailed"));
    }
  } finally {
    if (selectedAddon.value?.id === requestedAddonId) {
      detailsLoading.value = false;
    }
  }
}

function statusLabel(status: AddonStatus) {
  const labels: Record<AddonStatus, string> = {
    current: t("statusCurrent"),
    update: t("statusUpdate"),
    untracked: t("statusUntracked"),
    checking: t("statusChecking"),
    updating: t("statusUpdating"),
    error: t("statusError"),
  };
  return labels[status];
}

function sourceLabel(source: AddonSource) {
  return {
    curseforge: "CurseForge",
    wowinterface: t("sourceWowInterface"),
    unknown: t("sourceLocal"),
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
  if (!date) return t("notPerformed");
  return date.toLocaleTimeString(locale.value, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatNumber(value: number) {
  return new Intl.NumberFormat(locale.value, { notation: "compact" }).format(
    value,
  );
}

function formatDate(value: string | undefined) {
  if (!value) return t("unknown");
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return t("unknown");
  return new Intl.DateTimeFormat(locale.value, {
    year: "numeric",
    month: "short",
    day: "numeric",
  }).format(date);
}

function displayValue(value: string | undefined, fallback: TranslationKey) {
  return value?.trim() ? value : t(fallback);
}

function errorMessage(error: unknown, fallback: string) {
  if (locale.value !== "zh-CN") return fallback;
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

watch(
  locale,
  (value) => {
    document.documentElement.lang = value;
  },
  { immediate: true },
);
</script>

<template>
  <n-config-provider
    :theme-overrides="themeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
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
              <div class="brand-caption">{{ t("addonManager") }}</div>
            </div>
          </div>

          <div class="sidebar-section-label">{{ t("gameVersions") }}</div>
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
                <strong>{{ gameFlavorLabel(installation.flavor) }}</strong>
                <small>{{ t("addonCount", { count: installation.addonCount }) }}</small>
              </span>
              <ChevronRight class="nav-chevron" :size="16" />
            </button>
          </nav>

          <div v-if="!loadingInstallations && !installations.length" class="no-game">
            <FolderSearch :size="22" />
            <span>{{ t("noGameClient") }}</span>
            <button type="button" @click="openSettings">
              {{ t("manualSelect") }}
            </button>
          </div>

          <div class="sidebar-spacer" />

          <button class="sidebar-settings" type="button" @click="openSettings">
            <Settings2 :size="18" />
            <span>{{ t("settings") }}</span>
          </button>
        </aside>

        <main class="main">
          <header class="topbar">
            <div class="topbar-title">
              <div class="eyebrow">
                <span class="online-dot" />
                {{ t("wowClient") }}
              </div>
              <h1>{{ activeInstallation ? gameFlavorLabel(activeInstallation.flavor) : t("addonManager") }}</h1>
            </div>
            <div class="topbar-actions">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button
                    quaternary
                    circle
                    :loading="scanning"
                    :disabled="!activeInstallation"
                    :aria-label="t('rescan')"
                    @click="runScan()"
                  >
                    <template #icon><RefreshCw :size="18" /></template>
                  </n-button>
                </template>
                {{ t("rescanAddonFolder") }}
              </n-tooltip>
              <n-button
                secondary
                :loading="checking"
                :disabled="!addons.length"
                @click="runUpdateCheck"
              >
                <template #icon><Sparkles :size="17" /></template>
                {{ t("checkUpdates") }}
              </n-button>
              <n-button
                type="primary"
                :loading="updatingAll"
                :disabled="!updateCount"
                @click="updateAll"
              >
                <template #icon><Download :size="17" /></template>
                {{ t("updateAll") }}
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
                  <span>{{ t("installedAddons") }}</span>
                  <strong>{{ addons.length }}</strong>
                </div>
                <small>{{ t("scannedAt", { time: formatRelativeTime(scanCompletedAt) }) }}</small>
              </article>
              <article class="summary-card">
                <div class="summary-icon coral">
                  <Download :size="21" />
                </div>
                <div>
                  <span>{{ t("updatesAvailable") }}</span>
                  <strong>{{ updateCount }}</strong>
                </div>
                <small>{{ checkedAt ? t("checkedAt", { time: formatRelativeTime(checkedAt) }) : t("waitingForCheck") }}</small>
              </article>
              <article class="summary-card">
                <div class="summary-icon mint">
                  <HardDrive :size="21" />
                </div>
                <div>
                  <span>{{ t("currentDirectory") }}</span>
                  <strong class="folder-value">{{ activeInstallation?.productFolder ?? "—" }}</strong>
                </div>
                <n-tooltip v-if="activeInstallation" trigger="hover">
                  <template #trigger>
                    <small class="path-text">{{ activeInstallation.addonsPath }}</small>
                  </template>
                  {{ activeInstallation.addonsPath }}
                </n-tooltip>
                <small v-else>{{ t("chooseGameDirectory") }}</small>
              </article>
            </section>

            <section class="addons-panel">
              <div class="panel-toolbar">
                <div>
                  <h2>{{ t("myAddons") }}</h2>
                  <p>{{ t("manageAddons") }}</p>
                </div>
                <div class="toolbar-controls">
                  <n-input
                    v-model:value="searchTerm"
                    clearable
                    :placeholder="t('searchAddons')"
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
                <span>{{ t("addon") }}</span>
                <span>{{ t("source") }}</span>
                <span>{{ t("localVersion") }}</span>
                <span>{{ t("status") }}</span>
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
                          v-if="addon.interfaceVersion"
                          size="tiny"
                          :bordered="false"
                          class="interface-tag"
                        >
                          {{ addon.interfaceVersion }}
                        </n-tag>
                      </div>
                      <p>{{ addon.notes || t("createdBy", { author: displayValue(addon.author, "unknownAuthor") }) }}</p>
                    </div>
                  </div>

                  <div class="source-cell">
                    <span class="source-mark" :data-source="addon.source">
                      {{ sourceMark(addon.source) }}
                    </span>
                    <span>{{ sourceLabel(addon.source) }}</span>
                  </div>

                  <div class="version-cell">
                    <span>{{ displayValue(addon.version, "unknown") }}</span>
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
                      {{ t("update") }}
                    </n-button>
                    <n-button
                      v-else
                      quaternary
                      size="small"
                      @click="showDetails(addon)"
                    >
                      {{ t("details") }}
                    </n-button>
                  </div>
                </article>
              </div>

              <n-empty
                v-else
                class="empty-state"
                :description="
                  addons.length ? t('noMatchingAddons') : t('noAddonsFound')
                "
              >
                <template #icon><Puzzle :size="42" :stroke-width="1.3" /></template>
                <template #extra>
                  <n-button v-if="activeInstallation" secondary @click="runScan()">
                    {{ t("rescan") }}
                  </n-button>
                  <n-button v-else type="primary" @click="openSettings">
                    {{ t("select") }}
                  </n-button>
                </template>
              </n-empty>
            </section>

            <footer class="content-footer">
              <span><ShieldCheck :size="14" /> {{ t("localOnly") }}</span>
              <span>WowBox 0.1.0</span>
            </footer>
          </div>
        </main>

        <n-modal
          v-model:show="settingsVisible"
          preset="card"
          :title="t('settings')"
          class="settings-modal"
          :bordered="false"
          :mask-closable="false"
          :close-on-esc="false"
        >
          <div class="settings-section">
            <div class="settings-section-title">
              <Settings2 :size="18" />
              <div>
                <strong>{{ t("language") }}</strong>
                <span>{{ t("languageHelp") }}</span>
              </div>
            </div>
            <n-form-item :label="t('language')">
              <n-select
                v-model:value="settingsDraft.language"
                :options="languageOptions"
              />
            </n-form-item>
          </div>

          <div class="settings-divider" />

          <div class="settings-section">
            <div class="settings-section-title">
              <FolderOpen :size="18" />
              <div>
                <strong>{{ t("gameDirectory") }}</strong>
                <span>{{ t("gameDirectoryHelp") }}</span>
              </div>
            </div>
            <n-form-item :label="t('gameRoot')">
              <div class="path-picker">
                <n-input
                  v-model:value="settingsDraft.gameRoot"
                  readonly
                  :placeholder="t('blankAutoDetect')"
                />
                <n-button secondary @click="selectGameRoot">{{ t("select") }}</n-button>
                <n-button
                  v-if="settingsDraft.gameRoot"
                  quaternary
                  @click="settingsDraft.gameRoot = ''"
                >
                  {{ t("auto") }}
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
                    <strong>{{ t("clientPaths") }}</strong>
                    <span>{{ t("clientPathsHelp") }}</span>
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
                        :placeholder="t('autoDetectedPath')"
                      />
                      <n-button
                        secondary
                        size="small"
                        @click="selectClientPath(option.flavor, option.label)"
                      >
                        {{ t("select") }}
                      </n-button>
                      <n-button
                        v-if="settingsDraft.clientPaths[option.flavor]"
                        quaternary
                        size="small"
                        @click="clearClientPath(option.flavor)"
                      >
                        {{ t("clear") }}
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
                <strong>{{ t("dataSources") }}</strong>
                <span>{{ t("dataSourcesHelp") }}</span>
              </div>
            </div>
            <n-form-item :label="t('source')">
              <n-select
                v-model:value="settingsDraft.pluginDataSource"
                :options="pluginDataSourceOptions"
              />
            </n-form-item>
            <n-form-item
              v-if="settingsDraft.pluginDataSource === 'curseforge'"
              :label="t('personalApiKey')"
            >
              <n-input
                v-model:value="settingsDraft.curseforgeApiKey"
                :type="apiKeyVisible ? 'text' : 'password'"
                :placeholder="t('defaultApiKey')"
              >
                <template #suffix>
                  <button
                    class="input-icon-button"
                    type="button"
                    :aria-label="apiKeyVisible ? t('hideKey') : t('showKey')"
                    @click="apiKeyVisible = !apiKeyVisible"
                  >
                    <EyeOff v-if="apiKeyVisible" :size="16" />
                    <Eye v-else :size="16" />
                  </button>
                </template>
              </n-input>
            </n-form-item>
            <div
              v-if="settingsDraft.pluginDataSource === 'curseforge'"
              class="setting-toggle"
            >
              <div>
                <strong>{{ t("rememberKey") }}</strong>
                <span>{{ t("rememberKeyHelp") }}</span>
              </div>
              <n-switch v-model:value="settingsDraft.rememberApiKey" />
            </div>
            <div class="setting-toggle">
              <div>
                <strong>{{ t("checkOnLaunch") }}</strong>
                <span>{{ t("checkOnLaunchHelp") }}</span>
              </div>
              <n-switch v-model:value="settingsDraft.checkOnLaunch" />
            </div>
          </div>

          <div class="privacy-note">
            <Info :size="16" />
            <span>{{ t("privacy") }}</span>
          </div>

          <template #footer>
            <div class="modal-footer">
              <n-button @click="cancelSettings">{{ t("cancel") }}</n-button>
              <n-button type="primary" @click="saveSettings">
                <template #icon><Check :size="16" /></template>
                {{ t("saveSettings") }}
              </n-button>
            </div>
          </template>
        </n-modal>

        <n-modal
          v-model:show="detailsVisible"
          preset="card"
          class="details-modal"
          :bordered="false"
          :title="t('addonDetails')"
        >
          <div v-if="selectedAddon" class="details-content">
            <div class="details-hero">
              <div class="addon-avatar large" :data-source="selectedAddon.source">
                {{ selectedAddon.title.slice(0, 1).toUpperCase() }}
              </div>
              <div>
                <h3>{{ selectedAddon.title }}</h3>
                <p>{{ selectedAddon.notes || t("noDescription") }}</p>
              </div>
            </div>
            <dl class="details-grid">
              <div><dt>{{ t("author") }}</dt><dd>{{ displayValue(selectedAddon.author, "unknown") }}</dd></div>
              <div><dt>{{ t("source") }}</dt><dd>{{ sourceLabel(selectedAddon.source) }}</dd></div>
              <div><dt>{{ t("localVersion") }}</dt><dd>{{ displayValue(selectedAddon.version, "unknown") }}</dd></div>
              <div><dt>{{ t("latestVersion") }}</dt><dd>{{ selectedAddon.latestVersion || t("notChecked") }}</dd></div>
              <div><dt>{{ t("interface") }}</dt><dd>{{ displayValue(selectedAddon.interfaceVersion, "unknown") }}</dd></div>
              <div><dt>{{ t("folderCount") }}</dt><dd>{{ selectedAddon.folders.length }}</dd></div>
              <template v-if="selectedAddon.remoteDetails">
                <div>
                  <dt>{{ t("curseForgeProjectId") }}</dt>
                  <dd>{{ selectedAddon.remoteDetails.projectId }}</dd>
                </div>
                <div>
                  <dt>{{ t("downloads") }}</dt>
                  <dd>{{ formatNumber(selectedAddon.remoteDetails.downloadCount) }}</dd>
                </div>
                <div>
                  <dt>{{ t("rating") }}</dt>
                  <dd>{{ selectedAddon.remoteDetails.rating?.toFixed(1) || t("unknown") }}</dd>
                </div>
                <div>
                  <dt>{{ t("thumbsUp") }}</dt>
                  <dd>{{ formatNumber(selectedAddon.remoteDetails.thumbsUpCount) }}</dd>
                </div>
                <div>
                  <dt>{{ t("dateCreated") }}</dt>
                  <dd>{{ formatDate(selectedAddon.remoteDetails.dateCreated) }}</dd>
                </div>
                <div>
                  <dt>{{ t("dateModified") }}</dt>
                  <dd>{{ formatDate(selectedAddon.remoteDetails.dateModified) }}</dd>
                </div>
              </template>
            </dl>

            <div v-if="detailsLoading" class="details-loading">
              <n-skeleton text :repeat="4" />
            </div>
            <div v-else-if="detailsError" class="error-note">
              <AlertCircle :size="16" />
              {{ detailsError }}
            </div>

            <template v-if="selectedAddon.remoteDetails">
              <div class="details-section">
                <span>{{ t("categories") }}</span>
                <div class="details-tags">
                  <n-tag
                    v-for="category in selectedAddon.remoteDetails.categories"
                    :key="category"
                    size="small"
                    :bordered="false"
                  >
                    {{ category }}
                  </n-tag>
                  <small v-if="!selectedAddon.remoteDetails.categories.length">
                    {{ t("noCategories") }}
                  </small>
                </div>
              </div>
              <div class="details-section">
                <span>{{ t("description") }}</span>
                <p class="remote-description">
                  {{ selectedAddon.remoteDetails.description || t("noDescription") }}
                </p>
              </div>
              <div
                v-if="selectedAddon.remoteDetails.websiteUrl"
                class="details-section"
              >
                <span>{{ t("projectWebsite") }}</span>
                <code class="project-url">
                  {{ selectedAddon.remoteDetails.websiteUrl }}
                </code>
              </div>
            </template>

            <div class="folder-list">
              <span>{{ t("folders") }}</span>
              <code v-for="folder in selectedAddon.folders" :key="folder">{{ folder }}</code>
            </div>
            <div v-if="selectedAddon.error" class="error-note">
              <AlertCircle :size="16" />
              {{ errorMessage(selectedAddon.error, t("checkUpdatesFailed")) }}
            </div>
          </div>
        </n-modal>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>
