import { useEffect, useState } from "react";
import TitleBar from "./components/TitleBar";
import StatusBar from "./components/StatusBar";
import ErrorModal from "./components/ErrorModal";
import { ChevronRight } from "./components/icons";
import ModeCard from "./panels/ModeCard";
import DifficultyCard from "./panels/DifficultyCard";
import PresetsPanel from "./panels/PresetsPanel";
import BotItemsPanel from "./panels/BotItemsPanel";
import CommandsPanel from "./panels/CommandsPanel";
import SettingsView from "./panels/settings/SettingsView";
import FirstRunLanguages from "./panels/settings/FirstRunLanguages";
import { useStore } from "./state/store";
import { useT, type I18nKey } from "./i18n";
import "./App.css";

type View = "main" | "settings" | "presets" | "botItems" | "commands";

const VIEWS: View[] = ["main", "settings", "presets", "botItems", "commands"];
const VIEW_KEY = "cs2bi.view";

export default function App() {
  const { error, clearError, ready, config } = useStore();
  const t = useT();
  // Remember the open view within a session (survives a webview reload), while a
  // fresh launch still starts on the main screen — sessionStorage clears on close.
  const [view, setView] = useState<View>(() => {
    const saved = sessionStorage.getItem(VIEW_KEY) as View | null;
    return saved && VIEWS.includes(saved) ? saved : "main";
  });
  useEffect(() => {
    sessionStorage.setItem(VIEW_KEY, view);
  }, [view]);
  const firstRun = ready && !!config && !config.first_run_done;

  const toMain = () => setView("main");

  const NAV: { view: View; key: I18nKey }[] = [
    { view: "presets", key: "pre.title" },
    { view: "botItems", key: "bi.title" },
    { view: "commands", key: "cmd.title" },
  ];

  return (
    <div className="shell">
      <TitleBar
        onSettings={() => setView((v) => (v === "settings" ? "main" : "settings"))}
      />

      {view === "settings" ? (
        <SettingsView onClose={toMain} />
      ) : view === "presets" ? (
        <PresetsPanel onBack={toMain} />
      ) : view === "botItems" ? (
        <BotItemsPanel onBack={toMain} />
      ) : view === "commands" ? (
        <CommandsPanel onBack={toMain} />
      ) : (
        <div className="shell__scroll">
          <StatusBar />
          <ModeCard />
          <DifficultyCard />
          {NAV.map(({ view: v, key }) => (
            <button
              key={v}
              className="nav-block glass"
              onClick={() => setView(v)}
            >
              <span className="nav-block__title">{t(key)}</span>
              <ChevronRight size={20} />
            </button>
          ))}
        </div>
      )}

      <ErrorModal error={error} onClose={clearError} />
      {firstRun && <FirstRunLanguages />}
    </div>
  );
}
