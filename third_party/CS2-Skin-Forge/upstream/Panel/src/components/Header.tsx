import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useT } from "../i18n";

interface HeaderProps {
  onSettingsClick: () => void;
  onAboutClick: () => void;
}

let cachedVersion: string | null = null;

/** Read the app version from Tauri (single source of truth: tauri.conf.json). */
export function useAppVersion(): string {
  const [version, setVersion] = useState<string>(cachedVersion ?? "");
  useEffect(() => {
    if (cachedVersion) return;
    getVersion()
      .then((v) => {
        cachedVersion = v;
        setVersion(v);
      })
      .catch(() => setVersion(""));
  }, []);
  return version;
}

export default function Header({ onSettingsClick, onAboutClick }: HeaderProps) {
  const { t } = useT();
  const version = useAppVersion();

  return (
    <header className="bg-black/30 border-b border-white/[0.06] px-4 py-2.5" data-tauri-drag-region>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-amber-400 to-orange-600 flex items-center justify-center shadow-lg shadow-amber-500/20">
            <svg className="w-5 h-5 text-black/80" fill="none" stroke="currentColor" strokeWidth={2.5} viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="4" />
              <path strokeLinecap="round" d="M12 2v4M12 18v4M2 12h4M18 12h4" />
            </svg>
          </div>
          <div>
            <div className="flex items-baseline gap-2">
              <h1 className="text-base font-bold text-white leading-tight">{t("app.title")}</h1>
              {version && <span className="text-[11px] text-gray-500 font-mono">v{version}</span>}
            </div>
            <p className="text-[11px] text-gray-500 leading-tight">{t("app.localOnly")} · Open Source</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className="chip !text-emerald-300 !border-emerald-500/25 !bg-emerald-500/[0.08]">
            {t("app.vacSafe")}
          </span>
          <button
            onClick={onSettingsClick}
            className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/[0.07] transition-colors"
            title={t("settings.title")}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
          <button
            onClick={onAboutClick}
            className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/[0.07] transition-colors"
            title={t("about.title")}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </button>
        </div>
      </div>
    </header>
  );
}
