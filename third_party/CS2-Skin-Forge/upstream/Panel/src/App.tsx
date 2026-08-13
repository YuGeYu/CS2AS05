import { useState, useEffect } from 'react';
import Header from './components/Header';
import TabNavigation from './components/TabNavigation';
import WeaponPanel from './components/WeaponPanel';
import KnifePanel from './components/KnifePanel';
import GlovePanel from './components/GlovePanel';
import AgentPanel from './components/AgentPanel';
import MusicKitPanel from './components/MusicKitPanel';
import PreviewPanel from './components/PreviewPanel';
import StatusBar from './components/StatusBar';
import SettingsPanel from './components/SettingsPanel';
import AboutDialog from './components/AboutDialog';
import DisclaimerDialog from './components/DisclaimerDialog';
import TutorialDialog from './components/TutorialDialog';
import UpdateBanner from './components/UpdateBanner';
import Modal from './components/ui/Modal';
import { Loadout } from './utils/types';
import { useT } from './i18n';
import { api, type AppConfig, type PluginCheckResult, type UpdateCheckResult } from './lib/api';
import { openUrl } from "@tauri-apps/plugin-opener";

const TUTORIAL_KEY = 'cs2skinmod.tutorial.done';
const UPDATE_DISMISS_KEY = 'cs2skinmod.update.dismissed';

const defaultLoadout: Loadout = {
  weaponPaints: {},
  weaponWears: {},
  weaponSeeds: {},
  weaponPaintsCt: {},
  weaponWearsCt: {},
  weaponSeedsCt: {},
  weaponPaintsT: {},
  weaponWearsT: {},
  weaponSeedsT: {},
  weaponStickers: {},
  weaponKeychains: {},
  weaponNametags: {},
  weaponStatTrak: {},
  knifeIndex: -1,
  knifePaint: -1,
  knifeWear: 0.01,
  knifeSeed: 0,
  knifeIndexCt: -1,
  knifePaintCt: -1,
  knifeWearCt: 0.01,
  knifeSeedCt: 0,
  knifeIndexT: -1,
  knifePaintT: -1,
  knifeWearT: 0.01,
  knifeSeedT: 0,
  gloveIndexCt: -1,
  glovePaintCt: -1,
  gloveWearCt: 0.01,
  gloveSeedCt: 0,
  gloveDefIndexCt: 0,
  gloveIndexT: -1,
  glovePaintT: -1,
  gloveWearT: 0.01,
  gloveSeedT: 0,
  gloveDefIndexT: 0,
  agentModelCt: -1,
  agentModelT: -1,
  agentModelPathCt: '',
  agentModelPathT: '',
  musicKit: -1,
  useRandom: true,
};

/** Migrate a saved loadout: copy legacy shared weapon/knife fields into the
 *  per-team fields when the per-team fields are still empty (pre-v1.6.0 saves). */
function migrateLoadout(saved: Partial<Loadout>): Partial<Loadout> {
  const merged = { ...saved };
  const hasTeamPaints =
    Object.keys(saved.weaponPaintsCt ?? {}).length > 0 ||
    Object.keys(saved.weaponPaintsT ?? {}).length > 0;
  if (!hasTeamPaints && Object.keys(saved.weaponPaints ?? {}).length > 0) {
    merged.weaponPaintsCt = { ...saved.weaponPaints };
    merged.weaponPaintsT = { ...saved.weaponPaints };
    merged.weaponWearsCt = { ...(saved.weaponWears ?? {}) };
    merged.weaponWearsT = { ...(saved.weaponWears ?? {}) };
    merged.weaponSeedsCt = { ...(saved.weaponSeeds ?? {}) };
    merged.weaponSeedsT = { ...(saved.weaponSeeds ?? {}) };
  }
  const hasTeamKnife = (saved.knifeIndexCt ?? -1) >= 0 || (saved.knifeIndexT ?? -1) >= 0;
  if (!hasTeamKnife && (saved.knifeIndex ?? -1) >= 0) {
    merged.knifeIndexCt = saved.knifeIndex!;
    merged.knifeIndexT = saved.knifeIndex!;
    merged.knifePaintCt = saved.knifePaint ?? -1;
    merged.knifePaintT = saved.knifePaint ?? -1;
    merged.knifeWearCt = saved.knifeWear ?? 0.01;
    merged.knifeWearT = saved.knifeWear ?? 0.01;
    merged.knifeSeedCt = saved.knifeSeed ?? 0;
    merged.knifeSeedT = saved.knifeSeed ?? 0;
  }
  return merged;
}

function App() {
  const { t } = useT();
  const [activeTab, setActiveTab] = useState('weapons');
  const [showSettings, setShowSettings] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const [showTutorial, setShowTutorial] = useState(false);
  const [_config, setConfig] = useState<AppConfig | null>(null);
  const [loadout, setLoadout] = useState<Loadout>({ ...defaultLoadout });
  const [isLoading, setIsLoading] = useState(true);
  const [update, setUpdate] = useState<UpdateCheckResult | null>(null);

  const [status, setStatus] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [showPluginWarning, setShowPluginWarning] = useState(false);
  const [pluginCheckResult, setPluginCheckResult] = useState<PluginCheckResult | null>(null);

  // Load config and saved loadout on mount
  useEffect(() => {
    const init = async () => {
      try {
        const cfg = await api.getConfig();
        setConfig(cfg);
        // If CS2 path is not configured, auto-open settings as a setup reminder
        if (!cfg.cs2Path) {
          setShowSettings(true);
        } else {
          // Check if plugin files are present and up-to-date
          try {
            const check = await api.checkPluginFiles();
            setPluginCheckResult(check);
            if (!check.allPresent || check.versionMismatch) {
              setShowPluginWarning(true);
            }
          } catch (e) {
            console.error("Plugin check failed:", e);
          }
        }
        // Try to load saved loadout for slot 0 (local player)
        const saved = await api.loadLoadout(0);
        if (saved) {
          setLoadout(prev => ({ ...prev, ...migrateLoadout(saved) }));
        }
      } catch (e) {
        console.error("Failed to initialize:", e);
      }

      // Non-blocking update check against the latest GitHub release
      try {
        const result = await api.checkUpdate();
        const dismissed = localStorage.getItem(UPDATE_DISMISS_KEY);
        if (result.updateAvailable && dismissed !== result.latestVersion) {
          setUpdate(result);
        }
      } catch (e) {
        console.warn("Update check failed:", e);
      }
      setIsLoading(false);
    };
    init();
  }, []);

  const updateLoadout = (updates: Partial<Loadout>) => {
    setLoadout(prev => ({ ...prev, ...updates }));
  };

  const handleApply = async () => {
    try {
      // Mirror the CT per-team values into the legacy shared fields so an
      // older deployed plugin still shows a sensible loadout.
      const payload: Loadout = {
        ...loadout,
        weaponPaints: { ...loadout.weaponPaintsCt },
        weaponWears: { ...loadout.weaponWearsCt },
        weaponSeeds: { ...loadout.weaponSeedsCt },
        knifeIndex: loadout.knifeIndexCt,
        knifePaint: loadout.knifePaintCt,
        knifeWear: loadout.knifeWearCt,
        knifeSeed: loadout.knifeSeedCt,
      };
      await api.saveLoadout(0, payload);
      setStatus({ message: t("status.applied"), type: 'success' });
    } catch (e) {
      console.error("Failed to save loadout:", e);
      setStatus({ message: t("status.error"), type: 'error' });
    }
    setTimeout(() => setStatus(null), 3000);
  };

  const handleReset = () => {
    setLoadout({ ...defaultLoadout });
    setStatus({ message: t("status.reset"), type: 'info' });
    setTimeout(() => setStatus(null), 3000);
  };

  const handleDeployFromWarning = () => {
    setShowPluginWarning(false);
    setShowSettings(true);
  };

  const handleDismissPluginWarning = () => {
    setShowPluginWarning(false);
  };

  const handleDismissUpdate = () => {
    if (update) {
      try { localStorage.setItem(UPDATE_DISMISS_KEY, update.latestVersion); } catch {}
    }
    setUpdate(null);
  };

  const handleDisclaimerAccepted = () => {
    // First real launch: show the tutorial right after the disclaimer
    try {
      if (!localStorage.getItem(TUTORIAL_KEY)) {
        setShowTutorial(true);
      }
    } catch {
      setShowTutorial(true);
    }
  };

  const handleTutorialClose = () => {
    try { localStorage.setItem(TUTORIAL_KEY, 'true'); } catch {}
    setShowTutorial(false);
  };

  const renderPanel = () => {
    switch (activeTab) {
      case 'weapons':
        return <WeaponPanel loadout={loadout} updateLoadout={updateLoadout} />;
      case 'knives':
        return <KnifePanel loadout={loadout} updateLoadout={updateLoadout} />;
      case 'gloves':
        return <GlovePanel loadout={loadout} updateLoadout={updateLoadout} />;
      case 'agents':
        return <AgentPanel loadout={loadout} updateLoadout={updateLoadout} />;
      case 'music':
        return <MusicKitPanel loadout={loadout} updateLoadout={updateLoadout} />;
      default:
        return <WeaponPanel loadout={loadout} updateLoadout={updateLoadout} />;
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden" data-tauri-drag-region>
      <Header onSettingsClick={() => setShowSettings(true)} onAboutClick={() => setShowAbout(true)} />
      <UpdateBanner update={update} onDismiss={handleDismissUpdate} />

      <main className="flex-1 max-w-7xl mx-auto w-full px-4 py-4 overflow-hidden">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-3">
              <div className="w-8 h-8 border-2 border-amber-500/30 border-t-amber-500 rounded-full animate-spin mx-auto" />
              <p className="text-sm text-gray-500">{t("common.loading")}</p>
            </div>
          </div>
        ) : (
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 h-full">
          <div className="lg:col-span-3 space-y-3 overflow-hidden flex flex-col">
            <TabNavigation activeTab={activeTab} setActiveTab={setActiveTab} />
            <div className="flex-1 overflow-y-auto pr-1.5">
              {renderPanel()}
            </div>
          </div>

          <div className="space-y-3 overflow-y-auto">
            <PreviewPanel loadout={loadout} />
            <div className="card space-y-2.5">
              <button onClick={handleApply} className="btn-primary w-full">
                {t("btn.apply")}
              </button>
              <button onClick={handleReset} className="btn-secondary w-full">
                {t("btn.reset")}
              </button>
            </div>
          </div>
        </div>
        )}
      </main>

      <StatusBar status={status} />

      <footer className="text-center text-[10px] text-gray-600 py-1.5 border-t border-white/[0.04]">
        <span
          onClick={() => openUrl("https://github.com/kaecho/CS2-Skin-Forge")}
          className="hover:text-gray-400 transition-colors cursor-pointer"
        >
          CS2 Skin Mod
        </span>
        <span className="mx-1">·</span>
        <span>Open Source & Free · 开源免费软件 · 请勿上当受骗</span>
      </footer>

      <SettingsPanel
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
        onConfigSaved={setConfig}
      />
      <AboutDialog
        isOpen={showAbout}
        onClose={() => setShowAbout(false)}
        onOpenTutorial={() => setShowTutorial(true)}
      />
      <TutorialDialog isOpen={showTutorial} onClose={handleTutorialClose} />
      <DisclaimerDialog onAccepted={handleDisclaimerAccepted} />

      {/* Plugin Warning Dialog */}
      {showPluginWarning && pluginCheckResult && (
        <Modal
          size="sm"
          title={<><span className="text-xl">⚠️</span>{t("setup.pluginWarning")}</>}
          onClose={handleDismissPluginWarning}
          footer={
            <div className="flex gap-3">
              <button onClick={handleDismissPluginWarning} className="btn-secondary flex-1">
                {t("setup.remindLater")}
              </button>
              <button onClick={handleDeployFromWarning} className="btn-primary flex-1">
                {t("setup.deployNow")}
              </button>
            </div>
          }
        >
          <div className="space-y-3">
            <p className="text-sm text-gray-300">
              {pluginCheckResult.missingFiles.length > 0
                ? t("status.pluginMissing")
                : pluginCheckResult.versionMismatch
                  ? t("status.pluginVersionMismatch")
                  : !pluginCheckResult.counterstrikesharpInstalled
                    ? t("setup.cssMissing")
                    : t("setup.pluginWarningMessage")}
            </p>
            {pluginCheckResult.missingFiles.length > 0 && (
              <div className="text-xs text-red-300 bg-red-500/[0.08] border border-red-500/20 rounded-lg p-2.5">
                {t("setup.pluginWarningMessage")}
                <ul className="list-disc list-inside mt-1">
                  {pluginCheckResult.missingFiles.map(f => <li key={f}>{f}</li>)}
                </ul>
              </div>
            )}
            {pluginCheckResult.versionMismatch && (
              <div className="text-xs text-amber-300 bg-amber-500/[0.08] border border-amber-500/20 rounded-lg p-2.5">
                {t("status.pluginVersionMismatch")}
                <br />
                Panel: v{pluginCheckResult.panelVersion} | Deployed: {pluginCheckResult.deployedVersion || t("common.unknown")}
              </div>
            )}
            {!pluginCheckResult.counterstrikesharpInstalled && (
              <div className="text-xs text-amber-300 bg-amber-500/[0.08] border border-amber-500/20 rounded-lg p-2.5">
                {t("setup.cssAutoInstall")}
              </div>
            )}
          </div>
        </Modal>
      )}
    </div>
  );
}

export default App;
