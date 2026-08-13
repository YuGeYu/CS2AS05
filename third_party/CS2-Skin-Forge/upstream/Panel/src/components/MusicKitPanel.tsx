import { Loadout } from '../utils/types';
import { musicKits } from '../data/skins';
import { getLocalizedName, musicKitNameMap } from '../data/localNames';
import { useT } from '../i18n';

interface MusicKitPanelProps {
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
}

export default function MusicKitPanel({ loadout, updateLoadout }: MusicKitPanelProps) {
  const { t, lang } = useT();

  const handleMusicKitSelect = (kitId: number) => {
    updateLoadout({ musicKit: kitId, useRandom: false });
  };

  const handleRandom = () => {
    updateLoadout({ musicKit: -1, useRandom: true });
  };

  return (
    <div className="space-y-3">
      <button
        onClick={handleRandom}
        className={`card card-hover w-full text-center py-3 ${loadout.musicKit === -1 ? 'card-selected' : ''}`}
      >
        <div className="text-sm font-semibold text-white">{t("preview.random")}</div>
        <div className="text-xs text-gray-500 mt-0.5">{t("music.title")}</div>
      </button>

      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
        {musicKits.map(kit => (
          <button
            key={kit.id}
            onClick={() => handleMusicKitSelect(kit.id)}
            className={`card card-hover !p-2.5 text-left ${loadout.musicKit === kit.id ? 'card-selected' : ''}`}
          >
            <div className="flex items-center space-x-2">
              {kit.image ? (
                <img src={kit.image} alt={kit.name}
                  className="w-10 h-10 object-contain rounded"
                  onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
              ) : (
                <div className="w-8 h-8 rounded-md bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center text-white text-sm">
                  ♪
                </div>
              )}
              <div className="text-xs font-medium text-white truncate">{getLocalizedName('music_kit-' + kit.id, musicKitNameMap, lang, kit.name)}</div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
