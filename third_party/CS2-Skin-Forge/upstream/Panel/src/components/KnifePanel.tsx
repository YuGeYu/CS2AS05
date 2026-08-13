import { useState } from 'react';
import { Loadout, Team } from '../utils/types';
import { knives, getKnifeImageUrl } from '../data/knives';
import { knifeSkinsByType } from '../data/knifeSkins';
import { useT } from '../i18n';
import TeamToggle from './TeamToggle';
import KnifeEditorModal from './editors/KnifeEditorModal';

interface KnifePanelProps {
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
}

/**
 * Knife overview: pick a knife type per team. Clicking a knife selects it
 * for the active team and opens the full-size skin editor modal.
 */
export default function KnifePanel({ loadout, updateLoadout }: KnifePanelProps) {
  const { t, lang } = useT();
  const isChinese = lang === 'schinese' || lang === 'tchinese';
  const [team, setTeam] = useState<Team>('ct');
  const [editorOpen, setEditorOpen] = useState(false);

  const indexKey = team === 'ct' ? 'knifeIndexCt' : 'knifeIndexT';
  const paintKey = team === 'ct' ? 'knifePaintCt' : 'knifePaintT';
  const currentIndex = loadout[indexKey];

  const handleKnifeClick = (index: number) => {
    if (currentIndex !== index) {
      // Keep the current paint if the new knife also has it (e.g. Fade
      // exists on most knives), otherwise fall back to random (-1).
      const curPaint = loadout[paintKey];
      const newDefindex = knives[index].defindex;
      const paintStillValid =
        curPaint >= 0 && (knifeSkinsByType[newDefindex] ?? []).some(s => s.id === curPaint);
      updateLoadout({
        [indexKey]: index,
        [paintKey]: paintStillValid ? curPaint : -1,
        useRandom: false,
      } as Partial<Loadout>);
    }
    setEditorOpen(true);
  };

  /** Team switch inside the editor: carry the viewed knife type over to the
   *  new team so the modal keeps editing the same knife. */
  const handleModalTeamChange = (newTeam: Team) => {
    const newIndexKey = newTeam === 'ct' ? 'knifeIndexCt' : 'knifeIndexT';
    const newPaintKey = newTeam === 'ct' ? 'knifePaintCt' : 'knifePaintT';
    if (loadout[newIndexKey] !== currentIndex) {
      const newDefindex = knives[currentIndex]?.defindex;
      const otherPaint = loadout[newPaintKey];
      const paintStillValid =
        otherPaint >= 0 && newDefindex !== undefined &&
        (knifeSkinsByType[newDefindex] ?? []).some(s => s.id === otherPaint);
      updateLoadout({
        [newIndexKey]: currentIndex,
        [newPaintKey]: paintStillValid ? otherPaint : -1,
        useRandom: false,
      } as Partial<Loadout>);
    }
    setTeam(newTeam);
  };

  const handleRandom = () => {
    updateLoadout({
      knifeIndexCt: -1, knifePaintCt: -1,
      knifeIndexT: -1, knifePaintT: -1,
      useRandom: true,
    });
  };

  const getKnifeName = (idx: number) => {
    if (idx < 0 || idx >= knives.length) return t('preview.random');
    return isChinese ? knives[idx].nameZh : knives[idx].name;
  };

  const getSelectedSkinName = (idx: number, paint: number): string | null => {
    if (idx < 0 || paint < 0) return null;
    const defindex = knives[idx]?.defindex;
    const skin = defindex ? knifeSkinsByType[defindex]?.find(s => s.id === paint) : undefined;
    return skin?.name ?? `#${paint}`;
  };

  const ctSkin = getSelectedSkinName(loadout.knifeIndexCt, loadout.knifePaintCt);
  const tSkin = getSelectedSkinName(loadout.knifeIndexT, loadout.knifePaintT);

  return (
    <div className="space-y-3">
      {/* Team selector: knives are configured per team */}
      <TeamToggle team={team} onChange={setTeam} ctLabel={t('knife.ct')} tLabel={t('knife.t')} />

      {/* Current selections for both teams */}
      <div className="flex gap-2 text-xs">
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-sky-400 font-semibold">{t('knife.ct')}:</span>{' '}
          <span className="text-gray-200">{getKnifeName(loadout.knifeIndexCt)}</span>
          {ctSkin && <span className="text-amber-300/80"> · {ctSkin}</span>}
        </div>
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-orange-400 font-semibold">{t('knife.t')}:</span>{' '}
          <span className="text-gray-200">{getKnifeName(loadout.knifeIndexT)}</span>
          {tSkin && <span className="text-amber-300/80"> · {tSkin}</span>}
        </div>
      </div>

      <button
        onClick={handleRandom}
        className={`card card-hover w-full text-center py-3 ${
          loadout.knifeIndexCt === -1 && loadout.knifeIndexT === -1 ? 'card-selected' : ''
        }`}
      >
        <div className="text-sm font-semibold text-white">{t("preview.random")}</div>
        <div className="text-xs text-gray-500 mt-0.5">{t("knife.selectType")}</div>
      </button>

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        {knives.map((knife, index) => (
          <button
            key={knife.defindex}
            onClick={() => handleKnifeClick(index)}
            className={`card card-hover !p-3 text-center ${currentIndex === index ? 'card-selected' : ''}`}
          >
            <img
              src={getKnifeImageUrl(knife.defindex)}
              alt={knife.name}
              className="w-full h-12 object-contain mb-1 opacity-90"
              onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
            />
            <div className="text-xs font-semibold text-white">{isChinese ? knife.nameZh : knife.name}</div>
          </button>
        ))}
      </div>

      {/* Knife editor modal */}
      {editorOpen && currentIndex >= 0 && (
        <KnifeEditorModal
          knifeIndex={currentIndex}
          team={team}
          onTeamChange={handleModalTeamChange}
          loadout={loadout}
          updateLoadout={updateLoadout}
          onClose={() => setEditorOpen(false)}
        />
      )}
    </div>
  );
}
