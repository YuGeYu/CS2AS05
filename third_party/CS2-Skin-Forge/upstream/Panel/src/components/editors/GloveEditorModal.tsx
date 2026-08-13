import { useState } from 'react';
import { Loadout, Team } from '../../utils/types';
import { gloves, getGloveTypeImage } from '../../data/skins';
import { getGloveLocalizedName, getGlovePaintLocalizedName } from '../../data/localNames';
import { useT } from '../../i18n';
import Modal from '../ui/Modal';
import PickerGrid from '../ui/PickerGrid';
import TeamToggle from '../TeamToggle';
import WearSeedControls from '../WearSeedControls';

interface GloveEditorModalProps {
  /** Index into the gloves[] array. */
  gloveIndex: number;
  team: Team;
  onTeamChange: (team: Team) => void;
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
  onClose: () => void;
}

/**
 * Full-size glove paint editor. Note: gloves intentionally have no
 * "default / no paint" option — each glove type needs a paint kit made
 * for its own 3D model, otherwise textures distort (see CLAUDE.md).
 */
export default function GloveEditorModal({
  gloveIndex,
  team,
  onTeamChange,
  loadout,
  updateLoadout,
  onClose,
}: GloveEditorModalProps) {
  const { t, lang } = useT();
  const [copied, setCopied] = useState(false);

  const glove = gloves[gloveIndex];

  const paintKey = team === 'ct' ? 'glovePaintCt' : 'glovePaintT';
  const wearKey = team === 'ct' ? 'gloveWearCt' : 'gloveWearT';
  const seedKey = team === 'ct' ? 'gloveSeedCt' : 'gloveSeedT';
  const currentPaint = loadout[paintKey];
  const currentWear = loadout[wearKey];
  const currentSeed = loadout[seedKey];
  const selectedPaint = currentPaint >= 0 ? glove?.paints.find((p) => p.id === currentPaint) : undefined;

  const gloveName = glove ? getGloveLocalizedName(glove.defindex, glove.name, lang) : '?';

  const handlePaintSelect = (paintId: number) => {
    updateLoadout({
      [paintKey]: paintId,
      ...(team === 'ct' ? { gloveDefIndexCt: glove.defindex } : { gloveDefIndexT: glove.defindex }),
      useRandom: false,
    } as Partial<Loadout>);
  };

  const copyToOtherTeam = () => {
    const updates: Partial<Loadout> =
      team === 'ct'
        ? {
            gloveIndexT: gloveIndex,
            glovePaintT: currentPaint,
            gloveWearT: currentWear,
            gloveSeedT: currentSeed,
            gloveDefIndexT: glove.defindex,
          }
        : {
            gloveIndexCt: gloveIndex,
            glovePaintCt: currentPaint,
            gloveWearCt: currentWear,
            gloveSeedCt: currentSeed,
            gloveDefIndexCt: glove.defindex,
          };
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
            src={glove ? getGloveTypeImage(glove.defindex, glove.codename) : ''}
            alt=""
            className="w-8 h-8 object-contain"
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
          <span className="truncate">{gloveName}</span>
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
          <PickerGrid
            items={(glove?.paints ?? []).map((p) => ({
              id: p.id,
              name: getGlovePaintLocalizedName(glove.defindex, p.id, p.name, lang),
              image: p.image,
            }))}
            selectedId={currentPaint >= 0 ? currentPaint : null}
            onSelect={handlePaintSelect}
          />
        </div>
        <div className="space-y-3 lg:sticky lg:top-0">
          <div className="p-3 bg-black/20 rounded-lg border border-white/[0.06]">
            {selectedPaint ? (
              <div className="flex items-center gap-3">
                {selectedPaint.image && (
                  <img src={selectedPaint.image} alt={selectedPaint.name} className="w-16 h-16 object-contain rounded" />
                )}
                <div className="min-w-0">
                  <div className="text-xs font-semibold text-amber-200 truncate">
                    {getGlovePaintLocalizedName(glove.defindex, selectedPaint.id, selectedPaint.name, lang)}
                  </div>
                  <div className="text-[10px] text-amber-400/60">{t('picker.selectedSkin')}</div>
                </div>
              </div>
            ) : (
              <div className="text-xs text-gray-500 text-center py-3">{t('editor.noSkinSelected')}</div>
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
