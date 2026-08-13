import { useState } from 'react';
import { Loadout, Team } from '../../utils/types';
import { knives, getKnifeImageUrl } from '../../data/knives';
import { knifeSkinsByType } from '../../data/knifeSkins';
import { useT } from '../../i18n';
import Modal from '../ui/Modal';
import PickerGrid from '../ui/PickerGrid';
import TeamToggle from '../TeamToggle';
import WearSeedControls from '../WearSeedControls';

interface KnifeEditorModalProps {
  /** Index into the knives[] array. */
  knifeIndex: number;
  team: Team;
  onTeamChange: (team: Team) => void;
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
  onClose: () => void;
}

/**
 * Full-size knife skin editor. Opens when a knife card is clicked;
 * the knife type itself is already selected by the panel.
 */
export default function KnifeEditorModal({
  knifeIndex,
  team,
  onTeamChange,
  loadout,
  updateLoadout,
  onClose,
}: KnifeEditorModalProps) {
  const { t, lang } = useT();
  const [copied, setCopied] = useState(false);
  const isChinese = lang === 'schinese' || lang === 'tchinese';

  const knife = knives[knifeIndex];
  const skins = knife ? knifeSkinsByType[knife.defindex] ?? [] : [];

  const paintKey = team === 'ct' ? 'knifePaintCt' : 'knifePaintT';
  const wearKey = team === 'ct' ? 'knifeWearCt' : 'knifeWearT';
  const seedKey = team === 'ct' ? 'knifeSeedCt' : 'knifeSeedT';
  const currentPaint = loadout[paintKey];
  const currentWear = loadout[wearKey];
  const currentSeed = loadout[seedKey];
  const selectedSkin = currentPaint >= 0 ? skins.find((s) => s.id === currentPaint) : undefined;

  const knifeName = knife ? (isChinese ? knife.nameZh : knife.name) : '?';

  const handlePaintSelect = (paintId: number) => {
    updateLoadout({ [paintKey]: paintId, useRandom: false } as Partial<Loadout>);
  };

  /** -1 = random paint each spawn (plugin picks from its pool). */
  const clearPaint = () => {
    updateLoadout({ [paintKey]: -1 } as Partial<Loadout>);
  };

  const copyToOtherTeam = () => {
    const updates: Partial<Loadout> =
      team === 'ct'
        ? { knifeIndexT: knifeIndex, knifePaintT: currentPaint, knifeWearT: currentWear, knifeSeedT: currentSeed }
        : { knifeIndexCt: knifeIndex, knifePaintCt: currentPaint, knifeWearCt: currentWear, knifeSeedCt: currentSeed };
    updateLoadout(updates);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <Modal
      size="editor"
      onClose={onClose}
      title={
        <>
          <img
            src={knife ? getKnifeImageUrl(knife.defindex) : ''}
            alt=""
            className="w-10 h-7 object-contain"
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
          <span className="truncate">{knifeName}</span>
          <span className={`text-[11px] font-bold ${team === 'ct' ? 'text-sky-400' : 'text-orange-400'}`}>
            {team === 'ct' ? t('team.ct') : t('team.t')}
          </span>
        </>
      }
      headerExtra={
        <div className="w-36">
          <TeamToggle team={team} onChange={onTeamChange} ctLabel={t('team.ct')} tLabel={t('team.t')} />
        </div>
      }
      footer={
        <div className="flex items-center gap-3">
          <div className="flex-1" />
          <button onClick={copyToOtherTeam} className="btn-secondary !py-1.5 text-xs">
            {copied ? t('team.copied') : team === 'ct' ? t('team.copyToT') : t('team.copyToCt')}
          </button>
          <button onClick={onClose} className="btn-primary !py-1.5 text-xs">
            {t('common.done')}
          </button>
        </div>
      }
    >
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-start">
        <div className="lg:col-span-2">
          {skins.length > 0 ? (
            <PickerGrid
              items={skins.map((s) => ({ id: s.id, name: s.name, image: s.image }))}
              selectedId={currentPaint >= 0 ? currentPaint : null}
              onSelect={handlePaintSelect}
              onClear={clearPaint}
              clearLabel={t('preview.random')}
            />
          ) : (
            <div className="text-center text-sm text-gray-500 py-16">{t('knife.noSkins')}</div>
          )}
        </div>
        <div className="space-y-3 lg:sticky lg:top-0">
          <div className="p-3 bg-black/20 rounded-lg border border-white/[0.06]">
            {selectedSkin ? (
              <div className="flex items-center gap-3">
                {selectedSkin.image && (
                  <img src={selectedSkin.image} alt={selectedSkin.name} className="w-16 h-16 object-contain rounded" />
                )}
                <div className="min-w-0">
                  <div className="text-xs font-semibold text-amber-200 truncate">{selectedSkin.name}</div>
                  <div className="text-[10px] text-amber-400/60">{t('picker.selectedSkin')}</div>
                </div>
              </div>
            ) : (
              <div className="text-xs text-gray-500 text-center py-3">
                {currentPaint >= 0 ? `Paint #${currentPaint}` : t('preview.random')}
              </div>
            )}
          </div>
          <WearSeedControls
            wear={currentWear}
            seed={currentSeed}
            onWearChange={(w) => updateLoadout({ [wearKey]: w } as Partial<Loadout>)}
            onSeedChange={(s) => updateLoadout({ [seedKey]: s } as Partial<Loadout>)}
          />
        </div>
      </div>
    </Modal>
  );
}
