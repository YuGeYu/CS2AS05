import { useState } from 'react';
import { Loadout, Team } from '../utils/types';
import { gloves, getGloveTypeImage } from '../data/skins';
import { getGloveLocalizedName } from '../data/localNames';
import { useT } from '../i18n';
import TeamToggle from './TeamToggle';
import GloveEditorModal from './editors/GloveEditorModal';

interface GlovePanelProps {
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
}

/**
 * Glove overview: pick a glove type per team. Clicking a glove selects it
 * for the active team (validating the paint kit) and opens the editor modal.
 */
export default function GlovePanel({ loadout, updateLoadout }: GlovePanelProps) {
  const { t, lang } = useT();
  const [team, setTeam] = useState<Team>('ct');
  const [editorOpen, setEditorOpen] = useState(false);

  const indexKey = team === 'ct' ? 'gloveIndexCt' : 'gloveIndexT';
  const currentIndex = loadout[indexKey];

  /** Pick a paint for a glove type: keep the given paint if valid for the
   *  type, otherwise use the type's first paint. Gloves must never have an
   *  unset/mismatched paint — wrong paint kits distort the textures. */
  const validPaintFor = (gloveIdx: number, paint: number): number => {
    const glove = gloves[gloveIdx];
    if (!glove) return -1;
    return glove.paints.some(p => p.id === paint) ? paint : glove.paints[0].id;
  };

  const handleGloveClick = (index: number) => {
    if (currentIndex !== index) {
      const glove = gloves[index];
      const curPaint = team === 'ct' ? loadout.glovePaintCt : loadout.glovePaintT;
      const newPaint = validPaintFor(index, curPaint);
      updateLoadout({
        [indexKey]: index,
        ...(team === 'ct'
          ? { glovePaintCt: newPaint, gloveDefIndexCt: glove.defindex }
          : { glovePaintT: newPaint, gloveDefIndexT: glove.defindex }),
        useRandom: false,
      } as Partial<Loadout>);
    }
    setEditorOpen(true);
  };

  /** Team switch inside the editor: carry the viewed glove type over to the
   *  new team so the modal keeps editing the same glove. */
  const handleModalTeamChange = (newTeam: Team) => {
    const newIndexKey = newTeam === 'ct' ? 'gloveIndexCt' : 'gloveIndexT';
    if (loadout[newIndexKey] !== currentIndex) {
      const glove = gloves[currentIndex];
      const otherPaint = newTeam === 'ct' ? loadout.glovePaintCt : loadout.glovePaintT;
      const newPaint = validPaintFor(currentIndex, otherPaint);
      updateLoadout({
        [newIndexKey]: currentIndex,
        ...(newTeam === 'ct'
          ? { glovePaintCt: newPaint, gloveDefIndexCt: glove.defindex }
          : { glovePaintT: newPaint, gloveDefIndexT: glove.defindex }),
        useRandom: false,
      } as Partial<Loadout>);
    }
    setTeam(newTeam);
  };

  const handleRandom = () => {
    updateLoadout({
      gloveIndexCt: -1, glovePaintCt: -1,
      gloveIndexT: -1, glovePaintT: -1,
      useRandom: true,
    });
  };

  const getGloveName = (idx: number) => {
    if (idx < 0 || idx >= gloves.length) return t("preview.random");
    return getGloveLocalizedName(gloves[idx].defindex, gloves[idx].name, lang);
  };

  return (
    <div className="space-y-3">
      {/* Team toggle */}
      <TeamToggle team={team} onChange={setTeam} ctLabel={t("glove.ct")} tLabel={t("glove.t")} />

      {/* Current selections for both teams */}
      <div className="flex gap-2 text-xs">
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-sky-400 font-semibold">{t("glove.ct")}:</span>{' '}
          <span className="text-gray-200">{getGloveName(loadout.gloveIndexCt)}</span>
        </div>
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-orange-400 font-semibold">{t("glove.t")}:</span>{' '}
          <span className="text-gray-200">{getGloveName(loadout.gloveIndexT)}</span>
        </div>
      </div>

      <button
        onClick={handleRandom}
        className={`card card-hover w-full text-center py-3 ${
          loadout.gloveIndexCt === -1 && loadout.gloveIndexT === -1 ? 'card-selected' : ''
        }`}
      >
        <div className="text-sm font-semibold text-white">{t("preview.random")}</div>
        <div className="text-xs text-gray-500 mt-0.5">{t("glove.selectType")}</div>
      </button>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
        {gloves.map((glove, index) => (
          <button
            key={glove.defindex}
            onClick={() => handleGloveClick(index)}
            className={`card card-hover !p-3 text-left ${currentIndex === index ? 'card-selected' : ''}`}
          >
            <div className="flex items-center space-x-2">
              <img
                src={getGloveTypeImage(glove.defindex, glove.codename)}
                alt={glove.name}
                className="w-10 h-10 object-contain"
                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
              />
              <div>
                <div className="text-xs font-semibold text-white">
                  {getGloveLocalizedName(glove.defindex, glove.name, lang)}
                </div>
                <div className="text-xs text-gray-500">{glove.paints.length} {t("tab.gloves").toLowerCase()}</div>
              </div>
            </div>
          </button>
        ))}
      </div>

      {/* Glove editor modal */}
      {editorOpen && currentIndex >= 0 && (
        <GloveEditorModal
          gloveIndex={currentIndex}
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
