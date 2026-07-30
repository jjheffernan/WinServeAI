const { invoke } = window.__TAURI__.core;

/** @type {ReadonlySet<string>} */
const CANONICAL = new Set([
  "Stopped",
  "Starting",
  "Ready",
  "Failed",
  "Stopping",
  "Crashed",
]);

const badgeEl = document.getElementById("badge");
const endpointEl = document.getElementById("endpoint");
const errorEl = document.getElementById("error");
const btnStart = document.getElementById("btn-start");
const btnStop = document.getElementById("btn-stop");
const btnRestart = document.getElementById("btn-restart");
const btnRefresh = document.getElementById("btn-refresh");
const btnCopy = document.getElementById("btn-copy");
const logView = document.getElementById("log-view");
const logDirEl = document.getElementById("log-dir");
const tabButtons = Array.from(document.querySelectorAll(".tabs [data-stream]"));
const settingsForm = document.getElementById("settings-form");
const btnSaveSettings = document.getElementById("btn-save-settings");
const btnReloadSettings = document.getElementById("btn-reload-settings");
const settingsNote = document.getElementById("settings-note");
const settingsPath = document.getElementById("settings-path");
const setHost = document.getElementById("set-host");
const setPort = document.getElementById("set-port");
const setModel = document.getElementById("set-model");
const setGpuAuto = document.getElementById("set-gpu-auto");
const setGpuLayers = document.getElementById("set-gpu-layers");
const setContext = document.getElementById("set-context");
const setFa = document.getElementById("set-fa");
const btnBrowseModel = document.getElementById("btn-browse-model");

let busy = false;
let currentStatus = "Unknown";
let settingsEditable = false;
let activeStream = "server";
/** @type {{ server: string[], llama: string[], error: string[], dir?: string }} */
let logCache = { server: [], llama: [], error: [] };

function normalizeStatus(raw) {
  if (CANONICAL.has(raw)) return raw;
  // Never map a vague "Running" to Ready — surface as unknown for operators.
  if (raw === "Running") return "Unknown";
  return CANONICAL.has(raw) ? raw : "Unknown";
}

function applySnap(snap, { optimistic } = {}) {
  const status = normalizeStatus(snap.status || "Unknown");
  currentStatus = status;
  badgeEl.dataset.status = status;
  badgeEl.textContent = status;
  if (snap.endpoint) endpointEl.textContent = snap.endpoint;
  errorEl.textContent = snap.error || "";
  if (!optimistic) syncButtons();
}

function syncButtons() {
  const s = currentStatus;
  const transitional = s === "Starting" || s === "Stopping";
  btnStart.disabled = busy || transitional || s === "Ready";
  btnStop.disabled = busy || transitional || s === "Stopped";
  btnRestart.disabled = busy || transitional || s === "Stopped";
  btnRefresh.disabled = busy;
  btnCopy.disabled = !endpointEl.textContent || endpointEl.textContent === "…";
  syncSettingsEnabled();
}

function syncSettingsEnabled() {
  const enabled = settingsEditable && !busy;
  [
    setHost,
    setPort,
    setModel,
    setGpuAuto,
    setGpuLayers,
    setContext,
    setFa,
    btnSaveSettings,
    btnBrowseModel,
  ].forEach((el) => {
    el.disabled = !enabled;
  });
  if (!settingsEditable) {
    settingsNote.className = "warn";
    settingsNote.textContent =
      "Stop the server before editing settings (Starting / Ready / Stopping).";
  }
}

function fillSettings(dto) {
  setHost.value = dto.host ?? "";
  setPort.value = dto.port ?? 8080;
  setModel.value = dto.modelPath ?? "";
  setGpuAuto.checked = !!dto.gpuAuto;
  setGpuLayers.value = dto.gpuLayers ?? "auto";
  setContext.value = dto.context ?? 32768;
  setFa.checked = !!dto.flashAttention;
  settingsPath.textContent = dto.configPath || "";
  settingsEditable = !!dto.editable;
  if (settingsEditable) {
    settingsNote.className = "";
    settingsNote.textContent = "";
  }
  syncSettingsEnabled();
}

function readSettingsPatch() {
  return {
    host: setHost.value.trim(),
    port: Number(setPort.value),
    modelPath: setModel.value.trim(),
    gpuAuto: setGpuAuto.checked,
    gpuLayers: setGpuLayers.value.trim() || "auto",
    context: Number(setContext.value),
    flashAttention: setFa.checked,
  };
}

async function reloadSettings() {
  try {
    const dto = await invoke("manager_config_summary");
    fillSettings(dto);
  } catch (e) {
    settingsNote.className = "err";
    settingsNote.textContent = String(e);
  }
}

function renderLogs() {
  const lines = logCache[activeStream] || [];
  logView.textContent = lines.length ? lines.join("\n") : "(no lines yet)";
  logView.scrollTop = logView.scrollHeight;
  if (logCache.dir) logDirEl.textContent = logCache.dir;
}

async function refreshLogs() {
  try {
    const tail = await invoke("manager_logs", { maxLines: 200 });
    logCache = {
      server: tail.server || [],
      llama: tail.llama || [],
      error: tail.error || [],
      dir: tail.dir || "",
    };
    renderLogs();
  } catch (e) {
    logView.textContent = `log tail failed: ${e}`;
  }
}

async function refresh() {
  errorEl.textContent = "";
  try {
    const snap = await invoke("manager_status");
    applySnap(snap);
  } catch (e) {
    errorEl.textContent = String(e);
    syncButtons();
  }
  await refreshLogs();
  await reloadSettings();
}

async function run(cmd, optimisticStatus) {
  if (busy) return;
  busy = true;
  syncButtons();
  errorEl.textContent = "";
  if (optimisticStatus) {
    applySnap(
      { status: optimisticStatus, endpoint: endpointEl.textContent, error: null },
      { optimistic: true }
    );
  }
  try {
    const snap = await invoke(cmd);
    applySnap(snap);
  } catch (e) {
    errorEl.textContent = String(e);
    await refresh();
  } finally {
    busy = false;
    syncButtons();
    await refreshLogs();
    await reloadSettings();
  }
}

tabButtons.forEach((btn) => {
  btn.onclick = () => {
    activeStream = btn.dataset.stream;
    tabButtons.forEach((b) => b.classList.toggle("active", b === btn));
    renderLogs();
  };
});

btnStart.onclick = () => run("manager_start", "Starting");
btnStop.onclick = () => run("manager_stop", "Stopping");
btnRestart.onclick = () => run("manager_restart", "Starting");
btnRefresh.onclick = () => refresh();
btnCopy.onclick = async () => {
  const url = endpointEl.textContent;
  if (!url || url === "…") return;
  try {
    await navigator.clipboard.writeText(url);
  } catch (e) {
    errorEl.textContent = `copy failed: ${e}`;
  }
};

btnReloadSettings.onclick = () => reloadSettings();
btnBrowseModel.onclick = async () => {
  if (!settingsEditable || busy) return;
  settingsNote.className = "";
  settingsNote.textContent = "";
  try {
    const path = await invoke("pick_model_path");
    if (!path) return;
    setModel.value = path;
    settingsNote.className = "ok";
    settingsNote.textContent =
      "Selected .gguf path — click Save settings to write YAML.";
  } catch (e) {
    settingsNote.className = "err";
    settingsNote.textContent = String(e);
  }
};
settingsForm.onsubmit = async (ev) => {
  ev.preventDefault();
  if (!settingsEditable || busy) return;
  busy = true;
  syncButtons();
  settingsNote.className = "";
  settingsNote.textContent = "Saving…";
  try {
    const dto = await invoke("manager_apply_settings", {
      patch: readSettingsPatch(),
    });
    fillSettings(dto);
    settingsNote.className = "ok";
    settingsNote.textContent = "Settings saved to YAML.";
    const snap = await invoke("manager_status");
    applySnap(snap);
  } catch (e) {
    settingsNote.className = "err";
    settingsNote.textContent = String(e);
    await reloadSettings();
  } finally {
    busy = false;
    syncButtons();
    await refreshLogs();
  }
};

// Crash / external stop detection + log tail while the window is open.
setInterval(() => {
  if (!busy) refresh();
}, 2000);

refresh();
