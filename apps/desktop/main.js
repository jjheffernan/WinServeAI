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

let busy = false;
let currentStatus = "Unknown";
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

// Crash / external stop detection + log tail while the window is open.
setInterval(() => {
  if (!busy) refresh();
}, 2000);

refresh();
