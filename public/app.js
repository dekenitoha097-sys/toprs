const $ = (id) => document.getElementById(id);
let latestProcesses = [];

function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, (character) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  })[character]);
}

function formatBytes(value) {
  if (!Number.isFinite(Number(value))) return "—";
  const units = ["o", "Kio", "Mio", "Gio", "Tio"];
  let amount = Number(value);
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit++;
  }
  return `${amount.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function formatUptime(value) {
  const seconds = Math.max(0, Math.floor(value || 0));
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${days} j ${String(hours).padStart(2, "0")} h ${String(minutes).padStart(2, "0")} min`;
}

function setGauge(id, value) {
  const percentage = Math.max(0, Math.min(100, Number(value) || 0));
  const gauge = $(id);
  const arc = $(`${id.replace("-gauge", "")}-arc`);
  const angle = -90 + percentage * 1.8;
  gauge.style.setProperty("--gauge-angle", `${angle}deg`);
  arc.style.strokeDashoffset = `${142 - percentage * 1.42}`;
  gauge.querySelector(".gauge-value").textContent = `${percentage.toFixed(0)}%`;
}

function renderProcesses() {
  const query = $("process-search").value.trim().toLocaleLowerCase();
  const visible = latestProcesses.filter((process) =>
    String(process.pid).includes(query) ||
    process.name.toLocaleLowerCase().includes(query)
  );

  $("process-rows").innerHTML = visible.length
    ? visible.map((process) => `<tr>
        <td class="pid">${escapeHtml(process.pid)}</td>
        <td class="process-name" title="${escapeHtml(process.name)}">${escapeHtml(process.name)}</td>
        <td><span class="state">${escapeHtml(process.state)}</span></td>
        <td class="memory-cell">${formatBytes(process.memory_kb * 1024)}</td>
      </tr>`).join("")
    : '<tr><td colspan="4" class="empty">Aucun processus correspondant</td></tr>';
}

function renderInfoList(id, items, emptyMessage, renderItem) {
  $(id).innerHTML = items.length
    ? items.map(renderItem).join("")
    : `<div class="empty">${escapeHtml(emptyMessage)}</div>`;
}

function render(data) {
  const { cpu, memory, disk, system } = data;
  const memoryPercent = Number(memory.usage_percent) || 0;
  const diskPercent = disk.total_bytes
    ? disk.used_bytes / disk.total_bytes * 100
    : 0;
  const swapPercent = memory.swap_total_kb
    ? memory.swap_used_kb / memory.swap_total_kb * 100
    : 0;

  $("cpu-value").textContent = cpu.logical_cores;
  $("cpu-sub").textContent = cpu.model;

  setGauge("memory-gauge", memoryPercent);
  $("memory-value").textContent = `${memoryPercent.toFixed(1)} %`;
  $("memory-sub").textContent =
    `${formatBytes(memory.used_kb * 1024)} utilisés · ${formatBytes(memory.available_kb * 1024)} disponibles`;

  setGauge("disk-gauge", diskPercent);
  $("disk-value").textContent = `${diskPercent.toFixed(1)} %`;
  $("disk-sub").textContent =
    `${formatBytes(disk.used_bytes)} utilisés · ${formatBytes(disk.available_bytes)} libres`;

  setGauge("swap-gauge", swapPercent);
  $("swap-value").textContent = memory.swap_total_kb
    ? `${swapPercent.toFixed(1)} %`
    : "—";
  $("swap-sub").textContent = memory.swap_total_kb
    ? `${formatBytes(memory.swap_used_kb * 1024)} / ${formatBytes(memory.swap_total_kb * 1024)}`
    : "Aucun espace swap détecté";

  $("host-summary").textContent = `${system.hostname} · ${cpu.model}`;
  $("hostname").textContent = system.hostname;
  $("kernel").textContent = system.kernel;
  $("uptime").textContent = formatUptime(system.uptime_seconds);

  $("process-count").textContent = `${data.process_count} processus`;
  latestProcesses = data.processes;
  renderProcesses();

  renderInfoList("battery-list", data.batteries, "Aucune batterie détectée", (battery) => `
    <div class="info-row">
      <span>${escapeHtml(battery.name)}</span>
      <strong>${battery.capacity == null ? "—" : `${escapeHtml(battery.capacity)} %`} · ${escapeHtml(battery.status || "État inconnu")}</strong>
    </div>`);

  renderInfoList("network-list", data.network, "Aucune interface détectée", (interfaceInfo) => `
    <div class="info-row">
      <span>${escapeHtml(interfaceInfo.interface)}</span>
      <strong class="network-value">↓ ${formatBytes(interfaceInfo.received_bytes)} · ↑ ${formatBytes(interfaceInfo.transmitted_bytes)}</strong>
    </div>`);

  renderInfoList("temperature-list", data.temperatures, "Aucun capteur détecté", (sensor) => `
    <div class="info-row">
      <span>${escapeHtml(sensor.sensor)}</span>
      <strong class="temp-value">${Number(sensor.temperature_celsius).toFixed(1)} °C</strong>
    </div>`);

  $("live").classList.remove("error");
  $("live-label").textContent = "Système opérationnel";
  $("notice").hidden = true;
  $("updated").textContent = `Dernière mise à jour · ${new Date().toLocaleTimeString("fr-FR")}`;
}

async function refresh() {
  try {
    const response = await fetch("/api/system", { cache: "no-store" });
    if (!response.ok) throw new Error(`Le serveur a répondu ${response.status}`);
    render(await response.json());
  } catch (error) {
    $("live").classList.add("error");
    $("live-label").textContent = "Connexion interrompue";
    $("notice").textContent = `Impossible de récupérer les métriques : ${error.message}`;
    $("notice").hidden = false;
  }
}

$("refresh").addEventListener("click", refresh);
$("process-search").addEventListener("input", renderProcesses);
refresh();
window.setInterval(refresh, 2000);
