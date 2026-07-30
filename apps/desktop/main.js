const { invoke } = window.__TAURI__.core;

const statusEl = document.getElementById("status");
const endpointEl = document.getElementById("endpoint");
const errorEl = document.getElementById("error");

async function refresh() {
  errorEl.textContent = "";
  try {
    const snap = await invoke("manager_status");
    statusEl.textContent = snap.status;
    endpointEl.textContent = snap.endpoint;
    if (snap.error) errorEl.textContent = snap.error;
  } catch (e) {
    errorEl.textContent = String(e);
  }
}

async function run(cmd) {
  errorEl.textContent = "";
  try {
    const snap = await invoke(cmd);
    statusEl.textContent = snap.status;
    endpointEl.textContent = snap.endpoint;
    if (snap.error) errorEl.textContent = snap.error;
  } catch (e) {
    errorEl.textContent = String(e);
  }
}

document.getElementById("btn-start").onclick = () => run("manager_start");
document.getElementById("btn-stop").onclick = () => run("manager_stop");
document.getElementById("btn-restart").onclick = () => run("manager_restart");
document.getElementById("btn-refresh").onclick = () => refresh();

refresh();
