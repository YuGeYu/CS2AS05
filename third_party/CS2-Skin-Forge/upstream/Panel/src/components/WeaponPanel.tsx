import { useState } from 'react';
import { Loadout, Team } from '../utils/types';
import { weaponCategories, getWeaponsByCategory } from '../data/weapons';
import { weaponPaints } from '../data/skins';
import { getWeaponDefaultImage } from '../data/weaponImages';
import { skinNamesEn } from '../data/skinNamesEn';
import { useT } from '../i18n';
import TeamToggle from './TeamToggle';
import WeaponEditorModal from './editors/WeaponEditorModal';

interface WeaponPanelProps {
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
}

/**
 * Weapon overview: category filter + weapon grid. Clicking a weapon opens
 * the full-size editor modal (skin / stickers / keychain / details).
 */
export default function WeaponPanel({ loadout, updateLoadout }: WeaponPanelProps) {
  const { t, lang } = useT();
  const [team, setTeam] = useState<Team>('ct');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const [editingWeapon, setEditingWeapon] = useState<number | null>(null);

  const filteredWeapons = getWeaponsByCategory(selectedCategory);
  const isChinese = lang === 'schinese' || lang === 'tchinese';

  const paints = team === 'ct' ? loadout.weaponPaintsCt : loadout.weaponPaintsT;

  const getCategoryLabel = (category: string) => {
    const keyMap: Record<string, string> = {
      'All': 'weapon.all', 'Pistols': 'weapon.pistols', 'Rifles': 'weapon.rifles',
      'Snipers': 'weapon.snipers', 'SMGs': 'weapon.smgs', 'Heavy': 'weapon.heavy',
    };
    return t(keyMap[category] as any) || category;
  };

  const getPaintName = (defindex: number, paintId: number, fallbackName: string): string => {
    if (isChinese) return fallbackName;
    return skinNamesEn[defindex]?.[paintId] || fallbackName;
  };

  return (
    <div className="space-y-3">
      {/* Team selector: weapon skins are configured per team */}
      <TeamToggle team={team} onChange={setTeam} ctLabel={t('team.ct')} tLabel={t('team.t')} />

      <div className="flex flex-wrap gap-1.5">
        {weaponCategories.map(category => (
          <button key={category} onClick={() => setSelectedCategory(category)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors duration-150 ${
              selectedCategory === category
                ? 'bg-amber-500 text-black'
                : 'bg-white/[0.05] text-gray-300 hover:bg-white/10 border border-white/[0.06]'
            }`}>
            {getCategoryLabel(category)}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        {filteredWeapons.map(weapon => {
          const hasCt = loadout.weaponPaintsCt[weapon.defindex] !== undefined;
          const hasT = loadout.weaponPaintsT[weapon.defindex] !== undefined;
          const paintId = paints[weapon.defindex];
          const paint = paintId !== undefined
            ? weaponPaints[weapon.defindex]?.find(p => p.id === paintId)
            : undefined;
          return (
            <button key={weapon.defindex} onClick={() => setEditingWeapon(weapon.defindex)}
              className="card card-hover !p-2 text-left cursor-pointer overflow-hidden relative">
              {(hasCt || hasT) && (
                <div className="absolute top-1.5 right-1.5 flex gap-1">
                  {hasCt && <span className="w-2 h-2 rounded-full bg-sky-400" title="CT" />}
                  {hasT && <span className="w-2 h-2 rounded-full bg-orange-400" title="T" />}
                </div>
              )}
              {/* Show the selected skin for the active team, else the stock weapon */}
              <img src={paint?.image || getWeaponDefaultImage(weapon.defindex)} alt={weapon.name}
                className="w-full h-12 object-contain mb-1 opacity-90"
                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
              <div className="text-xs font-semibold text-white truncate">
                {isChinese ? weapon.nameZh : weapon.name}
              </div>
              <div className={`text-[10px] truncate ${paint ? 'text-amber-300/80' : 'text-gray-600'}`}>
                {paint ? getPaintName(weapon.defindex, paint.id, paint.name) : t('weapon.noPaint')}
              </div>
            </button>
          );
        })}
      </div>

      {/* Weapon editor modal */}
      {editingWeapon !== null && (
        <WeaponEditorModal
          defindex={editingWeapon}
          team={team}
          onTeamChange={setTeam}
          loadout={loadout}
          updateLoadout={updateLoadout}
          onClose={() => setEditingWeapon(null)}
        />
      )}
    </div>
  );
}
