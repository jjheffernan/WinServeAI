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

let busy = false;
let currentStatus = "Unknown";

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

async function refresh() {
  errorEl.textContent = "";
  try {
    const snap = await invoke("manager_status");
    applySnap(snap);
  } catch (e) {
    errorEl.textContent = String(e);
    syncButtons();
  }
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
  }
}

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

// Crash / external stop detection while the window is open.
setInterval(() => {
  if (!busy) refresh();
}, 2000);

refresh();
