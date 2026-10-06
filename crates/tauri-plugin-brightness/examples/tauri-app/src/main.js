// ============================================================
// Brightness Control — example application popup frontend.
// Vanilla ES module. No framework, no build step.
//
// Talks to the backend through window.__TAURI__.core.invoke (because
// withGlobalTauri: true in tauri.conf.json), calling the commands exposed by
// @tauri-apps/plugin-brightness, prefixed with `plugin:brightness|`.
//
// When opened in a plain browser (no __TAURI__), it falls back to a MOCK
// display list so the UI is testable standalone.
//
// The equivalent typed bindings are published as
// @tauri-apps/plugin-brightness, which the plugin's own README shows.
// ============================================================

// ---------- Constants ----------
const MOCK_MONITORS = [
  { id: "mock-0", name: "Monitor 1", brightness: 85, max: 100 },
  { id: "mock-1", name: "Monitor 2", brightness: 60, max: 100 },
];

const APP_NAME = "Brightness Control";
const APP_VERSION = "0.1.0";

/** Debounce window for slider -> set_brightness calls (ms). */
const SET_BRIGHTNESS_DEBOUNCE_MS = 80;

// ---------- Tauri API resolution (with mock fallback) ----------
const tauriGlobal = window.__TAURI__;
const isTauri = !!(
  tauriGlobal &&
  tauriGlobal.core &&
  typeof tauriGlobal.core.invoke === "function"
);

/**
 * Invoke a Tauri command, or fall back to a mock implementation
 * when running outside Tauri (plain browser, dev preview).
 * @param {string} cmd
 * @param {Record<string, unknown>} [args]
 * @returns {Promise<any>}
 */
const invoke = isTauri
  ? tauriGlobal.core.invoke
  : async (cmd, args = {}) => {
      console.log(`[MOCK] invoke('${cmd}', ${JSON.stringify(args)})`);
      switch (cmd) {
        case "plugin:brightness|list_monitors":
          return MOCK_MONITORS.map((m) => ({ ...m }));
        case "plugin:brightness|get_brightness": {
          const m = MOCK_MONITORS.find((x) => x.id === args.id);
          if (!m) throw new Error(`display \`${args.id}\` was not found`);
          return {
            id: m.id,
            brightness: m.brightness,
            max: m.max,
            brightnessPercent: Math.round((m.brightness / (m.max || 100)) * 100),
          };
        }
        case "plugin:brightness|set_brightness": {
          const m = MOCK_MONITORS.find((x) => x.id === args.id);
          if (m) m.brightness = args.value;
          return;
        }
        case "plugin:brightness|diagnose":
          // Return a plausible mock diagnostic so the UI is testable standalone.
          return {
            i2cDevLoaded: true,
            i2cDevSysfs: true,
            i2cDevices: ["/dev/i2c-0", "/dev/i2c-1"],
            inI2cGroup: true,
            inVideoGroup: true,
            ddcutilAvailable: true,
            ddcutilDetectOutput: "[mock] Display 1: DELL U2723QE",
            ddcHiCount: 2,
            ddcutilCount: 2,
            backlightCount: 1,
            hasNvidia: false,
            suggestions: [],
          };
        case "hide_window":
          console.log("[MOCK] would hide window");
          return;
        case "quit_app":
          console.log("[MOCK] would quit app");
          return;
        default:
          console.warn(`[MOCK] unknown command: ${cmd}`);
          return null;
      }
    };

/**
 * Listen for a Tauri event, or no-op when outside Tauri.
 * @returns {Promise<() => void>} unlisten function
 */
const listen = isTauri
  ? tauriGlobal.event.listen
  : async () => {
      // No-op unlisten
      return () => {};
    };

// ---------- DOM references (populated in init) ----------
let card;
let monitorList;
let viewMain;
let viewSettings;
let openSettingsBtn;
let backBtn;

// ---------- Utilities ----------

/**
 * Trailing-edge debounce.
 * @template T
 * @param {(...args: any[]) => T} fn
 * @param {number} wait
 */
function debounce(fn, wait) {
  let timer = null;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), wait);
  };
}

/**
 * Update the slider's filled-portion CSS variable so the track
 * gradient reflects the current thumb position.
 * @param {HTMLInputElement} input
 */
function updateSliderFill(input) {
  const min = Number(input.min) || 0;
  const max = Number(input.max) || 100;
  const val = Number(input.value);
  const pct = ((val - min) / (max - min)) * 100;
  input.style.setProperty("--fill", pct + "%");
}

// ---------- Rendering ----------

/**
 * Build a single monitor section (sun icon + name + slider) and
 * wire up its input/change handlers.
 * @param {{id: string, name: string, brightness: number, max: number}} monitor
 * @returns {HTMLElement}
 */
function createMonitorSection(monitor) {
  const section = document.createElement("div");
  section.className = "monitor";
  section.dataset.id = monitor.id;

  // Normalize brightness to a 0..100 percentage for the slider.
  // The slider is always 0..100; set_brightness receives the same number.
  const initialPct = Math.round(
    (monitor.brightness / (monitor.max || 100)) * 100
  );

  section.innerHTML = `
    <div class="monitor-row">
      <svg class="sun-icon" width="22" height="22" viewBox="0 0 24 24" fill="none"
           stroke="currentColor" stroke-width="2" stroke-linecap="round"
           stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="5"></circle>
        <line x1="12" y1="1" x2="12" y2="3"></line>
        <line x1="12" y1="21" x2="12" y2="23"></line>
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
        <line x1="1" y1="12" x2="3" y2="12"></line>
        <line x1="21" y1="12" x2="23" y2="12"></line>
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
      </svg>
      <span class="monitor-name"></span>
    </div>
    <input type="range" class="brightness-slider" min="0" max="100"
           value="${initialPct}" aria-label="Brightness for ${monitor.name}" />
  `;

  // Set the monitor name safely (avoid XSS from backend strings).
  section.querySelector(".monitor-name").textContent = monitor.name;

  const slider = section.querySelector(".brightness-slider");
  updateSliderFill(slider);

  // Debounced set_brightness — fires during drag, throttled to ~80ms.
  const debouncedSet = debounce((id, value) => {
    invoke("plugin:brightness|set_brightness", { id, value }).catch((err) =>
      console.error(`set_brightness(${id}, ${value}) failed:`, err)
    );
  }, SET_BRIGHTNESS_DEBOUNCE_MS);

  // Debounced reconcile — on `change` (release), re-read hardware value
  // and snap the slider to whatever the monitor actually reports.
  const debouncedReconcile = debounce(async (id) => {
    try {
      const actual = await invoke("plugin:brightness|get_brightness", { id });
      // The command reports both the raw value and the percentage, and the
      // slider always works in percentages.
      const snappedPct = Math.round(actual.brightnessPercent ?? 0);
      slider.value = String(snappedPct);
      updateSliderFill(slider);
    } catch (err) {
      console.error(`get_brightness(${id}) failed:`, err);
    }
  }, SET_BRIGHTNESS_DEBOUNCE_MS);

  // `input` fires continuously while dragging.
  slider.addEventListener("input", (e) => {
    updateSliderFill(e.target);
    debouncedSet(monitor.id, parseInt(e.target.value, 10));
  });

  // `change` fires once when the user releases the slider.
  slider.addEventListener("change", () => {
    debouncedReconcile(monitor.id);
  });

  return section;
}

/**
 * Fetch the monitor list from the backend (or mock) and render one
 * section per monitor.
 * @returns {Promise<void>}
 */
async function renderMonitors() {
  let monitors = [];
  try {
    monitors = await invoke("plugin:brightness|list_monitors");
    if (!Array.isArray(monitors) || monitors.length === 0) {
      throw new Error("no monitors returned");
    }
  } catch (err) {
    console.error("list_monitors failed, falling back to mock:", err);
    monitors = MOCK_MONITORS;
  }

  monitorList.innerHTML = "";

  // If the only "monitor" is the synthetic fallback, show a diagnostic panel
  // so the user knows WHY nothing is being detected.
  const isFallback =
    monitors.length === 1 && monitors[0].id === "fallback-0";
  if (isFallback) {
    monitorList.appendChild(createDiagnosticPanel(monitors[0]));
    return;
  }

  for (const m of monitors) {
    monitorList.appendChild(createMonitorSection(m));
  }
}

/**
 * Build a diagnostic panel shown when NO real monitors are detected.
 * Calls the backend `diagnose` command and renders actionable suggestions.
 * @returns {HTMLElement}
 */
function createDiagnosticPanel(fallbackMonitor) {
  const panel = document.createElement("div");
  panel.className = "diagnostic-panel";
  panel.style.cssText = "display:flex;flex-direction:column;gap:10px;";

  panel.innerHTML = `
    <div style="padding:10px 12px;border-radius:10px;background:rgba(255,180,80,0.25);border:1px solid rgba(255,160,60,0.5);font-size:12.5px;color:#5a3a10;line-height:1.5;">
      <strong>⚠ No DDC/CI display was detected.</strong><br>
      On a desktop with HDMI or DisplayPort monitors this is usually a missing
      <code>i2c-dev</code> module, a missing <code>i2c</code> group membership,
      or an NVIDIA driver that needs <code>ddcutil</code>.
    </div>
    <div id="diag-results" style="font-size:12px;color:#2c3e50;line-height:1.6;"></div>
    <button id="diag-refresh" type="button"
      style="align-self:flex-start;padding:6px 12px;border:none;border-radius:8px;background:#4a90e2;color:#fff;cursor:pointer;font-size:12px;">
      Run again
    </button>
  `;

  const results = panel.querySelector("#diag-results");
  const refresh = panel.querySelector("#diag-refresh");
  refresh.addEventListener("mousedown", (e) => e.stopPropagation());

  const runDiag = async () => {
    results.textContent = "Running diagnostics…";
    refresh.disabled = true;
    try {
      const d = await invoke("plugin:brightness|diagnose");
      const parts = [];
      parts.push(`<div><b>i2c-dev loaded:</b> ${d.i2cDevLoaded ? "✓ yes" : "✗ no"}</div>`);
      parts.push(`<div><b>i2c group:</b> ${d.inI2cGroup ? "✓ yes" : "✗ no"}</div>`);
      parts.push(`<div><b>/dev/i2c-* devices:</b> ${d.i2cDevices.length ? d.i2cDevices.join(", ") : "none"}</div>`);
      parts.push(`<div><b>ddcutil installed:</b> ${d.ddcutilAvailable ? "✓ yes" : "✗ no"}</div>`);
      parts.push(`<div><b>ddc-hi displays:</b> ${d.ddcHiCount}</div>`);
      parts.push(`<div><b>ddcutil displays:</b> ${d.ddcutilCount}</div>`);
      parts.push(`<div><b>Internal panels:</b> ${d.backlightCount}</div>`);
      parts.push(`<div><b>NVIDIA GPU:</b> ${d.hasNvidia ? "✓ yes (requires ddcutil)" : "no"}</div>`);
      if (d.suggestions.length) {
        parts.push(`<div style="margin-top:8px;padding-top:8px;border-top:1px solid rgba(0,0,0,0.1);"><b>Suggested fixes:</b><ul style="margin:6px 0 0;padding-left:18px;">${d.suggestions.map(s => `<li style="margin-bottom:4px;font-family:monospace;font-size:11px;">${s}</li>`).join("")}</ul></div>`);
      }
      results.innerHTML = parts.join("");
    } catch (err) {
      results.textContent = "Diagnostics failed: " + err;
    }
    refresh.disabled = false;
  };

  refresh.addEventListener("click", runDiag);
  // Auto-run once on first render.
  runDiag();

  return panel;
}

// ---------- View switching ----------

function showMainView() {
  viewMain.classList.add("active");
  viewSettings.classList.remove("active");
}

function showSettingsView() {
  viewMain.classList.remove("active");
  viewSettings.classList.add("active");
}

// ---------- Init ----------

function init() {
  // Grab DOM references.
  card = document.getElementById("card");
  monitorList = document.getElementById("monitor-list");
  viewMain = document.getElementById("view-main");
  viewSettings = document.getElementById("view-settings");
  openSettingsBtn = document.getElementById("open-settings-btn");
  backBtn = document.getElementById("back-btn");

  // --- Settings button (header) ---
  openSettingsBtn.addEventListener("click", showSettingsView);
  // Prevent the header's data-tauri-drag-region from triggering a window
  // drag when the user clicks the settings button.
  openSettingsBtn.addEventListener("mousedown", (e) => e.stopPropagation());

  // --- Back button (settings header) ---
  backBtn.addEventListener("click", showMainView);
  backBtn.addEventListener("mousedown", (e) => e.stopPropagation());

  // --- Click outside the card -> hide window ---
  // Anything that doesn't land on the card (or its descendants) is the
  // transparent window margin, treated as a "click away" gesture.
  document.addEventListener("click", (e) => {
    if (!card.contains(e.target)) {
      invoke("hide_window").catch((err) =>
        console.error("hide_window failed:", err)
      );
    }
  });

  // --- Escape -> hide window ---
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      invoke("hide_window").catch((err) =>
        console.error("hide_window failed:", err)
      );
    }
  });

  // --- Listen for "open-settings" event from the tray menu ---
  // The Rust backend emits this when the user picks "Settings" from the
  // system-tray context menu.
  listen("open-settings", () => {
    showSettingsView();
  }).catch((err) =>
    console.error("Failed to listen for open-settings event:", err)
  );

  // --- Populate About section ---
  const aboutName = document.getElementById("about-name");
  const aboutVersion = document.getElementById("about-version");
  if (aboutName) aboutName.textContent = APP_NAME;
  if (aboutVersion) aboutVersion.textContent = `v${APP_VERSION}`;

  // --- Wire settings controls (no-op persistence for now) ---
  const stepSelect = document.getElementById("step-select");
  if (stepSelect) {
    stepSelect.addEventListener("change", (e) => {
      console.log("Brightness step set to:", e.target.value);
    });
  }
  const toggleLogin = document.getElementById("toggle-login");
  if (toggleLogin) {
    toggleLogin.addEventListener("change", (e) => {
      console.log("Launch at login:", e.target.checked);
    });
  }
  const toggleDdcci = document.getElementById("toggle-ddcci");
  if (toggleDdcci) {
    toggleDdcci.addEventListener("change", (e) => {
      console.log("DDC/CI enabled:", e.target.checked);
    });
  }

  // --- Render the monitor list ---
  renderMonitors().catch((err) =>
    console.error("renderMonitors failed:", err)
  );

  console.log(
    `[brightness] frontend ready (mode: ${isTauri ? "tauri" : "mock"})`
  );
}

// Run init now if DOM is already parsed, else wait for DOMContentLoaded.
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
