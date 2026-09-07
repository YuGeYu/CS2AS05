import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  getCurrentWindow,
  currentMonitor,
  LogicalSize,
} from "@tauri-apps/api/window";
import App from "./App";
import { ToastProvider } from "./components/Toast";
import { AppStateProvider } from "./state/store";
import { applyGlassSupport } from "./lib/glass";
import "./styles/global.css";

// The UI is designed for a 460×720 logical-pixel viewport and scaled up to 125%
// (zoom 1.25 → a 575×900 window). On smaller screens a full 900px-tall window is
// clamped by the OS work area, which forced an internal scrollbar. So we fit the
// window to the monitor: pick the largest zoom (≤1.25) whose window still fits the
// available work area, size the window to exactly the scaled design (so the main
// view never needs to scroll), and centre it. The design always lays out at
// 460×720 regardless of zoom, so the content fits at any zoom we choose.
const DESIGN_W = 460;
const DESIGN_H = 720;
const MAX_ZOOM = 1.25;
const MIN_ZOOM = 0.7;

// The window is created hidden (visible:false). We size + centre it while still
// hidden, then add the `app-ready` class (kicks off the liquid entrance) and
// reveal it — so it appears already at its final size/position with no resize or
// recenter jump. `reveal` is idempotent and guarded by a timeout safety net so a
// hung/denied sizing call can never leave the window invisible.
let revealed = false;
function reveal() {
  if (revealed) return;
  revealed = true;
  document.documentElement.classList.add("app-ready");
  getCurrentWindow()
    .show()
    .catch(() => {});
}

async function fitWindowToScreen() {
  let zoom = MAX_ZOOM;
  try {
    const mon = await currentMonitor();
    if (mon) {
      const sf = mon.scaleFactor || 1;
      // Logical work area, leaving room for the taskbar and a small margin.
      const availH = mon.size.height / sf - 64;
      const availW = mon.size.width / sf - 32;
      zoom = Math.min(MAX_ZOOM, availH / DESIGN_H, availW / DESIGN_W);
      zoom = Math.max(MIN_ZOOM, zoom);
    }
  } catch {
    /* monitor unavailable — keep MAX_ZOOM and the default window size */
  }
  await getCurrentWebview()
    .setZoom(zoom)
    .catch(() => {});
  try {
    const win = getCurrentWindow();
    await win.setSize(
      new LogicalSize(Math.round(DESIGN_W * zoom), Math.round(DESIGN_H * zoom))
    );
    await win.center();
  } catch {
    /* sizing not permitted — fall back to the configured window size */
  }
  reveal();
}

fitWindowToScreen();
// Safety net: never leave the window hidden if sizing hangs or is denied.
setTimeout(reveal, 1200);

// Disable every right-click menu (native and custom). On some machines the
// custom menu's scrim could trap pointer events ("window looks frozen until you
// left-click"); suppressing the context menu entirely removes that class of bug.
document.addEventListener("contextmenu", (e) => e.preventDefault());

// Disable the in-page browser find (Ctrl/⌘+F and F3) — it has no place in a
// desktop control panel and confused users. Nothing else uses these keys.
document.addEventListener(
  "keydown",
  (e) => {
    const find =
      ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F")) ||
      e.key === "F3";
    if (find) e.preventDefault();
  },
  { capture: true }
);

applyGlassSupport();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <AppStateProvider>
        <App />
      </AppStateProvider>
    </ToastProvider>
  </React.StrictMode>
);
