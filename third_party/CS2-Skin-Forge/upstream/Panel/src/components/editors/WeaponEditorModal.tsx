import { useState } from 'react';
import { Loadout, Team, StickerInfo } from '../../utils/types';
import { weapons } from '../../data/weapons';
import { weaponPaints } from '../../data/skins';
import { allStickers, getStickerImageUrl } from '../../data/stickers';
import { allKeychains, getKeychainImageUrl, type KeychainData } from '../../data/keychains';
import { getWeaponDefaultImage } from '../../data/weaponImages';
import { skinNamesEn } from '../../data/skinNamesEn';
import { useT } from '../../i18n';
import Modal from '../ui/Modal';
import PickerGrid from '../ui/PickerGrid';
import TeamToggle from '../TeamToggle';
import WearSeedControls from '../WearSeedControls';

type EditorTab = 'skin' | 'stickers' | 'keychain' | 'details';

interface WeaponEditorModalProps {
  defindex: number;
  team: Team;
  onTeamChange: (team: Team) => void;
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
  onClose: () => void;
}

const EMPTY_STICKERS: StickerInfo[] = [{ id: 0 }, { id: 0 }, { id: 0 }, { id: 0 }, { id: 0 }];

function SliderField({
  label,
  min,
  max,
  step,
  value,
  onChange,
  format,
}: {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  onChange: (v: number) => void;
  format?: (v: number) => string;
}) {
  return (
    <div>
      <div className="flex items-center justify-between">
        <label className="text-[10px] text-gray-500">{label}</label>
        <span className="text-[10px] text-gray-500 tabular-nums">{format ? format(value) : value.toFixed(2)}</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="w-full h-1 accent-amber-500"
      />
    </div>
  );
}

/**
 * Full-size weapon editor. Opens when a weapon card is clicked and hosts
 * everything editable for one weapon: skin (per team), wear/seed, stickers,
 * keychain, nametag and StatTrak.
 */
export default function WeaponEditorModal({
  defindex,
  team,
  onTeamChange,
  loadout,
  updateLoadout,
  onClose,
}: WeaponEditorModalProps) {
  const { t, lang } = useT();
  const [tab, setTab] = useState<EditorTab>('skin');
  const [activeStickerSlot, setActiveStickerSlot] = useState(0);
  const [copied, setCopied] = useState(false);

  const isChinese = lang === 'schinese' || lang === 'tchinese';
  const weapon = weapons.find((w) => w.defindex === defindex);
  const paintsForWeapon = weaponPaints[defindex] ?? [];

  // Per-team map accessors
  const paintsKey = team === 'ct' ? 'weaponPaintsCt' : 'weaponPaintsT';
  const wearsKey = team === 'ct' ? 'weaponWearsCt' : 'weaponWearsT';
  const seedsKey = team === 'ct' ? 'weaponSeedsCt' : 'weaponSeedsT';
  const otherPaintsKey = team === 'ct' ? 'weaponPaintsT' : 'weaponPaintsCt';
  const otherWearsKey = team === 'ct' ? 'weaponWearsT' : 'weaponWearsCt';
  const otherSeedsKey = team === 'ct' ? 'weaponSeedsT' : 'weaponSeedsCt';
  const paints = loadout[paintsKey];
  const wears = loadout[wearsKey];
  const seeds = loadout[seedsKey];
  const currentPaintId = paints[defindex];
  const currentPaint = currentPaintId !== undefined ? paintsForWeapon.find((p) => p.id === currentPaintId) : undefined;

  const getPaintName = (paintId: number, fallbackName: string): string => {
    if (isChinese) return fallbackName;
    return skinNamesEn[defindex]?.[paintId] || fallbackName;
  };

  const translateStickerName = (name: string): string => {
    if (!isChinese) return name;
    return name
      .replace(/\(Holo\)/g, '(全息)')
      .replace(/\(Foil\)/g, '(箔金)')
      .replace(/\(Gold\)/g, '(金色)')
      .replace(/\(Glitter\)/g, '(闪亮)')
      .replace(/\(Paper\)/g, '(纸贴)')
      .replace(/\(Lenticular\)/g, '(光栅)');
  };

  const getKeychainName = (kc: KeychainData) => (isChinese ? kc.nameZh || kc.name : kc.name);

  const displayName = weapon ? (isChinese ? weapon.nameZh : weapon.name) : `#${defindex}`;

  // ── Skin handlers ────────────────────────────────────────────────

  const handlePaintSelect = (paintId: number) => {
    updateLoadout({
      [paintsKey]: { ...paints, [defindex]: paintId },
      useRandom: false,
    } as Partial<Loadout>);
  };

  /** Remove the active team's skin for this weapon (falls back to default in game). */
  const clearTeamPaint = () => {
    const removeKey = <T,>(map: Record<number, T>) => {
      const next = { ...map };
      delete next[defindex];
      return next;
    };
    const newPaints = removeKey(paints);
    const bothEmpty =
      Object.keys(newPaints).length === 0 && Object.keys(loadout[otherPaintsKey]).length === 0;
    updateLoadout({
      [paintsKey]: newPaints,
      [wearsKey]: removeKey(wears),
      [seedsKey]: removeKey(seeds),
      ...(bothEmpty ? { useRandom: true } : {}),
    } as Partial<Loadout>);
  };

  /** Reset everything configured for this weapon: both teams + cosmetics. */
  const resetWeapon = () => {
    const removeKey = <T,>(map: Record<number, T>) => {
      const next = { ...map };
      delete next[defindex];
      return next;
    };
    const newPaintsCt = removeKey(loadout.weaponPaintsCt);
    const newPaintsT = removeKey(loadout.weaponPaintsT);
    updateLoadout({
      weaponPaintsCt: newPaintsCt,
      weaponPaintsT: newPaintsT,
      weaponWearsCt: removeKey(loadout.weaponWearsCt),
      weaponWearsT: removeKey(loadout.weaponWearsT),
      weaponSeedsCt: removeKey(loadout.weaponSeedsCt),
      weaponSeedsT: removeKey(loadout.weaponSeedsT),
      weaponStickers: removeKey(loadout.weaponStickers),
      weaponKeychains: removeKey(loadout.weaponKeychains),
      weaponNametags: removeKey(loadout.weaponNametags),
      weaponStatTrak: removeKey(loadout.weaponStatTrak),
      useRandom: Object.keys(newPaintsCt).length === 0 && Object.keys(newPaintsT).length === 0,
    });
  };

  const copyToOtherTeam = () => {
    if (currentPaintId === undefined) return;
    updateLoadout({
      [otherPaintsKey]: { ...loadout[otherPaintsKey], [defindex]: currentPaintId },
      [otherWearsKey]: { ...loadout[otherWearsKey], [defindex]: wears[defindex] ?? 0.01 },
      [otherSeedsKey]: { ...loadout[otherSeedsKey], [defindex]: seeds[defindex] ?? 0 },
    } as Partial<Loadout>);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  // ── Sticker handlers ─────────────────────────────────────────────

  const stickers = loadout.weaponStickers[defindex] ?? EMPTY_STICKERS;
  const currentSticker = stickers[activeStickerSlot];

  const setSticker = (slot: number, info: StickerInfo) => {
    const next = [...(loadout.weaponStickers[defindex] ?? EMPTY_STICKERS)];
    next[slot] = info;
    updateLoadout({ weaponStickers: { ...loadout.weaponStickers, [defindex]: next } });
  };

  const handleStickerSelect = (stickerId: number) => {
    setSticker(activeStickerSlot, { id: stickerId, scale: 1, wear: 0, rotation: 0, offsetX: 0, offsetY: 0 });
  };

  const handleStickerField = (field: keyof StickerInfo, value: number) => {
    setSticker(activeStickerSlot, { ...currentSticker, [field]: value });
  };

  // ── Keychain handlers ────────────────────────────────────────────

  const keychain = loadout.weaponKeychains[defindex];

  const handleKeychainSelect = (keychainId: number) => {
    const existing = keychain ?? { id: 0, offsetX: 0, offsetY: 0, offsetZ: 0, seed: 0 };
    updateLoadout({
      weaponKeychains: { ...loadout.weaponKeychains, [defindex]: { ...existing, id: keychainId } },
    });
  };

  const clearKeychain = () => {
    const next = { ...loadout.weaponKeychains };
    delete next[defindex];
    updateLoadout({ weaponKeychains: next });
  };

  const handleKeychainField = (field: string, value: number) => {
    const existing = keychain ?? { id: 0, offsetX: 0, offsetY: 0, offsetZ: 0, seed: 0 };
    updateLoadout({
      weaponKeychains: { ...loadout.weaponKeychains, [defindex]: { ...existing, [field]: value } },
    });
  };

  // ── Details handlers ─────────────────────────────────────────────

  const handleNametagChange = (name: string) => {
    const next = { ...loadout.weaponNametags };
    if (name.trim() === '') delete next[defindex];
    else next[defindex] = name;
    updateLoadout({ weaponNametags: next });
  };

  const handleStatTrakToggle = () => {
    const next = { ...loadout.weaponStatTrak };
    if (next[defindex]?.enabled) delete next[defindex];
    else next[defindex] = { enabled: true, count: 0 };
    updateLoadout({ weaponStatTrak: next });
  };

  const handleStatTrakCount = (count: number) => {
    const existing = loadout.weaponStatTrak[defindex] ?? { enabled: true, count: 0 };
    updateLoadout({ weaponStatTrak: { ...loadout.weaponStatTrak, [defindex]: { ...existing, count } } });
  };

  // ── Tab metadata (with "configured" indicator dots) ──────────────

  const hasStickers = stickers.some((s) => s.id > 0);
  const hasKeychain = (keychain?.id ?? 0) > 0;
  const hasDetails = loadout.weaponNametags[defindex] !== undefined || loadout.weaponStatTrak[defindex]?.enabled;

  const tabs: { id: EditorTab; label: string; active?: boolean }[] = [
    { id: 'skin', label: t('editor.tabSkin'), active: currentPaintId !== undefined },
    { id: 'stickers', label: t('editor.tabStickers'), active: hasStickers },
    { id: 'keychain', label: t('editor.tabKeychain'), active: hasKeychain },
    { id: 'details', label: t('editor.tabDetails'), active: !!hasDetails },
  ];

  const teamBadge = (
    <span className={`text-[11px] font-bold ${team === 'ct' ? 'text-sky-400' : 'text-orange-400'}`}>
      {team === 'ct' ? t('team.ct') : t('team.t')}
    </span>
  );

  return (
    <Modal
      size="editor"
      onClose={onClose}
      title={
        <>
          <img
            src={getWeaponDefaultImage(defindex)}
            alt=""
            className="w-10 h-7 object-contain"
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
          <span className="truncate">{displayName}</span>
          {teamBadge}
        </>
      }
      headerExtra={
        <div className="w-36">
          <TeamToggle team={team} onChange={onTeamChange} ctLabel={t('team.ct')} tLabel={t('team.t')} />
        </div>
      }
      footer={
        <div className="flex items-center gap-3">
          <button onClick={resetWeapon} className="text-xs text-red-400 hover:text-red-300 transition-colors">
            {t('editor.resetItem')}
          </button>
          <div className="flex-1" />
          {currentPaintId !== undefined && (
            <button onClick={copyToOtherTeam} className="btn-secondary !py-1.5 text-xs">
              {copied ? t('team.copied') : team === 'ct' ? t('team.copyToT') : t('team.copyToCt')}
            </button>
          )}
          <button onClick={onClose} className="btn-primary !py-1.5 text-xs">
            {t('common.done')}
          </button>
        </div>
      }
    >
      {/* Tab bar */}
      <div className="flex gap-1 mb-4 bg-black/30 p-1 rounded-lg border border-white/[0.06] sticky -top-4 z-10 backdrop-blur-md">
        {tabs.map((tabInfo) => (
          <button
            key={tabInfo.id}
            onClick={() => setTab(tabInfo.id)}
            className={`relative flex-1 py-1.5 px-3 rounded-md text-xs font-semibold transition-colors ${
              tab === tabInfo.id
                ? 'bg-amber-500 text-black'
                : 'text-gray-400 hover:text-white hover:bg-white/[0.06]'
            }`}
          >
            {tabInfo.label}
            {tabInfo.active && tab !== tabInfo.id && (
              <span className="absolute top-1 right-1.5 w-1.5 h-1.5 bg-emerald-400 rounded-full" />
            )}
          </button>
        ))}
      </div>

      {/* ── Skin tab ── */}
      {tab === 'skin' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-start">
          <div className="lg:col-span-2">
            <PickerGrid
              items={paintsForWeapon.map((p) => ({ id: p.id, name: getPaintName(p.id, p.name), image: p.image }))}
              selectedId={currentPaintId ?? null}
              onSelect={handlePaintSelect}
              onClear={clearTeamPaint}
            />
          </div>
          <div className="space-y-3 lg:sticky lg:top-8">
            <div className="p-3 bg-black/20 rounded-lg border border-white/[0.06]">
              {currentPaint ? (
                <div className="flex items-center gap-3">
                  {currentPaint.image && (
                    <img src={currentPaint.image} alt={currentPaint.name} className="w-16 h-16 object-contain rounded" />
                  )}
                  <div className="min-w-0">
                    <div className="text-xs font-semibold text-amber-200 truncate">
                      {getPaintName(currentPaint.id, currentPaint.name)}
                    </div>
                    <div className="text-[10px] text-amber-400/60">{t('picker.selectedSkin')}</div>
                  </div>
                </div>
              ) : (
                <div className="text-xs text-gray-500 text-center py-3">{t('editor.noSkinSelected')}</div>
              )}
            </div>
            {currentPaintId !== undefined && (
              <WearSeedControls
                wear={wears[defindex] ?? 0.01}
                seed={seeds[defindex] ?? 0}
                onWearChange={(w) => updateLoadout({ [wearsKey]: { ...wears, [defindex]: w } } as Partial<Loadout>)}
                onSeedChange={(s) => updateLoadout({ [seedsKey]: { ...seeds, [defindex]: s } } as Partial<Loadout>)}
              />
            )}
          </div>
        </div>
      )}

      {/* ── Stickers tab ── */}
      {tab === 'stickers' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-start">
          <div className="lg:col-span-2">
            <PickerGrid
              items={allStickers.map((s) => ({
                id: s.id,
                name: translateStickerName(s.name),
                image: getStickerImageUrl(s.image),
              }))}
              selectedId={currentSticker?.id ?? null}
              onSelect={handleStickerSelect}
              searchPlaceholder={t('sticker.searchPlaceholder')}
              tile="sm"
              pageSize={100}
            />
          </div>
          <div className="space-y-3 lg:sticky lg:top-8">
            {/* Slot selector */}
            <div className="flex gap-1.5">
              {[0, 1, 2, 3, 4].map((slot) => {
                const filled = (stickers[slot]?.id ?? 0) > 0;
                return (
                  <button
                    key={slot}
                    onClick={() => setActiveStickerSlot(slot)}
                    className={`relative flex-1 py-1.5 rounded-md text-xs transition-colors ${
                      activeStickerSlot === slot
                        ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40'
                        : 'bg-white/[0.05] text-gray-400 hover:bg-white/10 border border-white/[0.06]'
                    }`}
                  >
                    {slot + 1}
                    {filled && <span className="absolute -top-1 -right-1 w-2 h-2 bg-emerald-400 rounded-full" />}
                  </button>
                );
              })}
            </div>

            {/* Current sticker in slot */}
            {currentSticker && currentSticker.id > 0 ? (
              (() => {
                const data = allStickers.find((s) => s.id === currentSticker.id);
                return (
                  <div className="flex items-center gap-2 p-2.5 bg-black/20 rounded-lg border border-white/[0.06]">
                    <img
                      src={getStickerImageUrl(data?.image || 'econ/stickers/community01')}
                      alt=""
                      className="w-10 h-10 object-contain"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = 'none';
                      }}
                    />
                    <span className="text-xs text-white flex-1 min-w-0 truncate">
                      {translateStickerName(data?.name || `#${currentSticker.id}`)}
                    </span>
                    <button
                      onClick={() => setSticker(activeStickerSlot, { id: 0 })}
                      className="text-xs text-red-400 hover:text-red-300 px-1"
                    >
                      ✕
                    </button>
                  </div>
                );
              })()
            ) : (
              <p className="text-xs text-gray-600 text-center py-2">{t('sticker.empty')}</p>
            )}

            {/* Position controls */}
            {currentSticker && currentSticker.id > 0 && (
              <div className="p-3 bg-black/20 rounded-lg border border-white/[0.06] space-y-2">
                <h5 className="text-xs font-semibold text-gray-300">{t('pos.title')}</h5>
                <SliderField label={t('pos.offsetX')} min={-2} max={2} step={0.01}
                  value={currentSticker.offsetX ?? 0} onChange={(v) => handleStickerField('offsetX', v)} />
                <SliderField label={t('pos.offsetY')} min={-2} max={2} step={0.01}
                  value={currentSticker.offsetY ?? 0} onChange={(v) => handleStickerField('offsetY', v)} />
                <SliderField label={t('pos.scale')} min={0.1} max={5} step={0.01}
                  value={currentSticker.scale ?? 1} onChange={(v) => handleStickerField('scale', v)} />
                <SliderField label={t('pos.rotation')} min={-180} max={180} step={1}
                  value={currentSticker.rotation ?? 0} onChange={(v) => handleStickerField('rotation', v)}
                  format={(v) => `${Math.round(v)}°`} />
                <SliderField label={t('pos.wear')} min={0} max={1} step={0.01}
                  value={currentSticker.wear ?? 0} onChange={(v) => handleStickerField('wear', v)} />
              </div>
            )}
          </div>
        </div>
      )}

      {/* ── Keychain tab ── */}
      {tab === 'keychain' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-start">
          <div className="lg:col-span-2">
            <PickerGrid
              items={allKeychains.map((k) => ({
                id: k.id,
                name: getKeychainName(k),
                image: getKeychainImageUrl(k.image),
              }))}
              selectedId={keychain?.id ?? null}
              onSelect={handleKeychainSelect}
              searchPlaceholder={t('keychain.searchPlaceholder')}
              tile="sm"
              pageSize={100}
            />
          </div>
          <div className="space-y-3 lg:sticky lg:top-8">
            {keychain && keychain.id > 0 ? (
              (() => {
                const data = allKeychains.find((k) => k.id === keychain.id);
                return (
                  <div className="flex items-center gap-2 p-2.5 bg-black/20 rounded-lg border border-white/[0.06]">
                    <img
                      src={getKeychainImageUrl(data?.image || '')}
                      alt=""
                      className="w-10 h-10 object-contain"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = 'none';
                      }}
                    />
                    <span className="text-xs text-white flex-1 min-w-0 truncate">
                      {data ? getKeychainName(data) : `#${keychain.id}`}
                    </span>
                    <button onClick={clearKeychain} className="text-xs text-red-400 hover:text-red-300 px-1">
                      ✕
                    </button>
                  </div>
                );
              })()
            ) : (
              <p className="text-xs text-gray-600 text-center py-2">{t('keychain.none')}</p>
            )}

            {keychain && keychain.id > 0 && (
              <div className="p-3 bg-black/20 rounded-lg border border-white/[0.06] space-y-2">
                <h5 className="text-xs font-semibold text-gray-300">{t('pos.title')}</h5>
                <SliderField label={t('pos.offsetX')} min={-5} max={5} step={0.01}
                  value={keychain.offsetX ?? 0} onChange={(v) => handleKeychainField('offsetX', v)} />
                <SliderField label={t('pos.offsetY')} min={-5} max={5} step={0.01}
                  value={keychain.offsetY ?? 0} onChange={(v) => handleKeychainField('offsetY', v)} />
                <SliderField label={t('pos.offsetZ')} min={-5} max={5} step={0.01}
                  value={keychain.offsetZ ?? 0} onChange={(v) => handleKeychainField('offsetZ', v)} />
                <div className="flex items-center justify-between pt-1">
                  <label className="text-[10px] text-gray-500">{t('pos.seed')}</label>
                  <input
                    type="number"
                    min={0}
                    max={9999}
                    value={keychain.seed ?? 0}
                    onChange={(e) => handleKeychainField('seed', parseInt(e.target.value) || 0)}
                    className="num-input"
                  />
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* ── Details tab ── */}
      {tab === 'details' && (
        <div className="max-w-md mx-auto space-y-4">
          <div className="p-4 bg-black/20 rounded-lg border border-white/[0.06] space-y-4">
            {/* Nametag */}
            <div>
              <label className="text-xs text-gray-400 block mb-1.5">{t('weapon.nametag')}</label>
              <input
                type="text"
                placeholder={t('weapon.nametagPlaceholder')}
                value={loadout.weaponNametags[defindex] ?? ''}
                onChange={(e) => handleNametagChange(e.target.value)}
                maxLength={64}
                className="input-field !text-xs"
              />
            </div>

            {/* StatTrak */}
            <div className="flex items-center justify-between">
              <label className="text-xs text-gray-400">{t('weapon.stattrak')}</label>
              <button
                onClick={handleStatTrakToggle}
                className={`px-3 py-1 rounded-md text-xs font-semibold transition-colors ${
                  loadout.weaponStatTrak[defindex]?.enabled
                    ? 'bg-orange-500/20 text-orange-300 border border-orange-500/40'
                    : 'bg-white/[0.05] text-gray-400 hover:bg-white/10 border border-white/[0.08]'
                }`}
              >
                {loadout.weaponStatTrak[defindex]?.enabled ? 'ON' : 'OFF'}
              </button>
            </div>
            {loadout.weaponStatTrak[defindex]?.enabled && (
              <div className="flex items-center justify-between">
                <label className="text-xs text-gray-400">{t('weapon.stattrakCount')}</label>
                <input
                  type="number"
                  min={0}
                  max={999999}
                  value={loadout.weaponStatTrak[defindex]?.count ?? 0}
                  onChange={(e) => handleStatTrakCount(parseInt(e.target.value) || 0)}
                  className="num-input !w-24"
                />
              </div>
            )}
          </div>
        </div>
      )}
    </Modal>
  );
}
