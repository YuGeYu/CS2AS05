import { useState } from "react";
import { useT } from "../i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../lib/api";
import { useAppVersion } from "./Header";
import Modal from "./ui/Modal";

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenTutorial: () => void;
}

const PROJECT_LINKS = [
  {
    name: "CS2-Skin-Forge",
    desc: "github.com/kaecho/CS2-Skin-Forge",
    url: "https://github.com/kaecho/CS2-Skin-Forge",
  },
  {
    name: "cs2-WeaponPaints",
    desc: "github.com/Nereziel/cs2-WeaponPaints",
    url: "https://github.com/Nereziel/cs2-WeaponPaints",
  },
  {
    name: "CounterStrikeSharp",
    desc: "github.com/roflmuffin/CounterStrikeSharp",
    url: "https://github.com/roflmuffin/CounterStrikeSharp",
  },
];

export default function AboutDialog({ isOpen, onClose, onOpenTutorial }: AboutDialogProps) {
  const { t } = useT();
  const version = useAppVersion();
  const [checking, setChecking] = useState(false);
  const [updateMsg, setUpdateMsg] = useState<{ text: string; url?: string } | null>(null);

  if (!isOpen) return null;

  const handleCheckUpdate = async () => {
    setChecking(true);
    setUpdateMsg(null);
    try {
      const result = await api.checkUpdate();
      if (result.updateAvailable) {
        setUpdateMsg({
          text: t("update.available", { version: result.latestVersion }),
          url: result.releaseUrl,
        });
      } else {
        setUpdateMsg({ text: t("update.upToDate") });
      }
    } catch {
      setUpdateMsg({ text: t("update.failed") });
    }
    setChecking(false);
  };

  return (
    <Modal
      size="sm"
      title={t("about.title")}
      onClose={onClose}
      footer={
        <button onClick={onClose} className="btn-primary w-full">
          {t("common.close")}
        </button>
      }
    >
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-11 h-11 rounded-xl bg-gradient-to-br from-amber-400 to-orange-600 flex items-center justify-center">
            <svg className="w-6 h-6 text-black/80" fill="none" stroke="currentColor" strokeWidth={2.5} viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="4" />
              <path strokeLinecap="round" d="M12 2v4M12 18v4M2 12h4M18 12h4" />
            </svg>
          </div>
          <div>
            <h3 className="text-base font-bold text-white">{t("app.title")}</h3>
            {version && <p className="text-xs text-gray-500 font-mono">v{version}</p>}
          </div>
        </div>

        <p className="text-sm text-gray-300">{t("about.description")}</p>

        {/* Update check */}
        <div className="space-y-2">
          <button onClick={handleCheckUpdate} disabled={checking} className="btn-secondary w-full text-sm">
            {checking ? t("update.checking") : t("update.check")}
          </button>
          {updateMsg && (
            <div className="flex items-center gap-2 text-xs text-gray-300 bg-black/20 rounded-lg px-3 py-2">
              <span className="flex-1">{updateMsg.text}</span>
              {updateMsg.url && (
                <button
                  onClick={() => openUrl(updateMsg.url!)}
                  className="text-amber-400 hover:text-amber-300 font-semibold shrink-0"
                >
                  {t("update.download")}
                </button>
              )}
            </div>
          )}
        </div>

        {/* Tutorial */}
        <button
          onClick={() => {
            onClose();
            onOpenTutorial();
          }}
          className="btn-secondary w-full text-sm"
        >
          {t("tutorial.open")}
        </button>

        {/* Credits & project links */}
        <div className="space-y-1.5">
          <p className="section-label">{t("about.credits")}</p>
          {PROJECT_LINKS.map((link) => (
            <button
              key={link.url}
              onClick={() => openUrl(link.url)}
              className="flex items-center gap-2.5 w-full px-3 py-2 rounded-lg bg-white/[0.04] border border-white/[0.06] hover:bg-white/[0.08] hover:border-white/[0.14] transition-colors text-left"
            >
              <svg className="w-4 h-4 text-gray-400 shrink-0" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
              </svg>
              <div className="flex-1 min-w-0">
                <div className="text-xs font-semibold text-white">{link.name}</div>
                <div className="text-[11px] text-gray-500 truncate">{link.desc}</div>
              </div>
              <svg className="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </button>
          ))}
        </div>
      </div>
    </Modal>
  );
}
