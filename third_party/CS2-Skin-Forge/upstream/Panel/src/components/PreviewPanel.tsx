import { Loadout } from '../utils/types';
import { knives } from '../data/knives';
import { gloves, musicKits, agentModels } from '../data/skins';
import { getLocalizedName, getGloveLocalizedName, agentNameMap, musicKitNameMap } from '../data/localNames';
import { useT } from '../i18n';

interface PreviewPanelProps {
  loadout: Loadout;
}

export default function PreviewPanel({ loadout }: PreviewPanelProps) {
  const { t, lang } = useT();
  const isChinese = lang === 'schinese' || lang === 'tchinese';

  const getKnifeName = (idx: number) => {
    if (idx === -1) return t("preview.random");
    const knife = knives[idx];
    if (!knife) return t("preview.notSelected");
    return isChinese ? knife.nameZh : knife.name;
  };

  const getGloveName = (idx: number) => {
    if (idx === -1) return t("preview.random");
    const glove = gloves[idx];
    if (!glove) return t("preview.notSelected");
    return getGloveLocalizedName(glove.defindex, glove.name, lang);
  };

  const getMusicKitName = () => {
    if (loadout.musicKit === -1) return t("preview.notSelected");
    const kit = musicKits.find(k => k.id === loadout.musicKit);
    if (!kit) return t("preview.notSelected");
    return getLocalizedName('music_kit-' + kit.id, musicKitNameMap, lang, kit.name);
  };

  const getAgentName = () => {
    const ctIdx = loadout.agentModelCt;
    const tIdx = loadout.agentModelT;
    const names: string[] = [];
    if (ctIdx >= 0 && ctIdx < agentModels.ct.length) {
      names.push(getLocalizedName('agent-' + agentModels.ct[ctIdx].id, agentNameMap, lang, agentModels.ct[ctIdx].name));
    }
    if (tIdx >= 0 && tIdx < agentModels.t.length) {
      names.push(getLocalizedName('agent-' + agentModels.t[tIdx].id, agentNameMap, lang, agentModels.t[tIdx].name));
    }
    if (names.length === 0) return t("preview.random");
    return names.join(' / ');
  };

  const ctWeaponCount = Object.keys(loadout.weaponPaintsCt).length;
  const tWeaponCount = Object.keys(loadout.weaponPaintsT).length;
  const hasCustomWeapons = ctWeaponCount > 0 || tWeaponCount > 0;

  const Row = ({ label, children }: { label: string; children: React.ReactNode }) => (
    <div>
      <div className="text-[11px] font-medium text-gray-500 mb-0.5">{label}</div>
      {children}
    </div>
  );

  return (
    <div className="card">
      <h3 className="text-sm font-semibold text-white mb-3">{t("preview.title")}</h3>

      <div className="space-y-3">
        <Row label={t("preview.knife")}>
          <div className="text-xs"><span className="text-sky-400 font-semibold">CT</span> <span className="text-gray-200">{getKnifeName(loadout.knifeIndexCt)}</span></div>
          <div className="text-xs"><span className="text-orange-400 font-semibold">T</span> <span className="text-gray-200">{getKnifeName(loadout.knifeIndexT)}</span></div>
        </Row>

        <Row label={t("preview.gloves")}>
          <div className="text-xs"><span className="text-sky-400 font-semibold">CT</span> <span className="text-gray-200">{getGloveName(loadout.gloveIndexCt)}</span></div>
          <div className="text-xs"><span className="text-orange-400 font-semibold">T</span> <span className="text-gray-200">{getGloveName(loadout.gloveIndexT)}</span></div>
        </Row>

        <Row label={t("agent.title")}>
          <div className="text-xs text-gray-200">{getAgentName()}</div>
        </Row>

        <Row label={t("preview.music")}>
          <div className="text-xs text-gray-200">{getMusicKitName()}</div>
        </Row>

        <Row label={t("preview.weapons")}>
          {hasCustomWeapons ? (
            <div className="text-xs text-gray-200">
              <span className="text-sky-400 font-semibold">CT</span> {ctWeaponCount}
              <span className="mx-1.5 text-gray-600">·</span>
              <span className="text-orange-400 font-semibold">T</span> {tWeaponCount}
            </div>
          ) : (
            <div className="text-xs text-gray-200">{t("preview.random")}</div>
          )}
        </Row>

        <div className="border-t border-white/[0.06] my-2"></div>

        <div className="text-center">
          <span className={`chip ${
            loadout.useRandom
              ? '!text-emerald-300 !border-emerald-500/25 !bg-emerald-500/[0.08]'
              : '!text-amber-300 !border-amber-500/25 !bg-amber-500/[0.08]'
          }`}>
            {loadout.useRandom ? t("preview.random") : t("preview.custom")}
          </span>
        </div>
      </div>
    </div>
  );
}
