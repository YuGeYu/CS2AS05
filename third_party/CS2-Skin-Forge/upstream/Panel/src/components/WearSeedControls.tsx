import { useT } from '../i18n';

interface WearSeedControlsProps {
  wear: number;
  seed: number;
  onWearChange: (wear: number) => void;
  onSeedChange: (seed: number) => void;
}

const WEAR_PRESETS = [
  { label: 'FN', value: 0.01 },
  { label: 'MW', value: 0.07 },
  { label: 'FT', value: 0.15 },
  { label: 'WW', value: 0.38 },
  { label: 'BS', value: 0.45 },
];

const clampWear = (v: number) => Math.min(1, Math.max(0, v));
const clampSeed = (v: number) => Math.min(1000, Math.max(0, Math.round(v)));

/**
 * Wear + pattern seed controls with both a slider and a manual numeric input.
 */
export default function WearSeedControls({ wear, seed, onWearChange, onSeedChange }: WearSeedControlsProps) {
  const { t } = useT();
  const activePreset = WEAR_PRESETS.find(p => Math.abs(p.value - wear) < 0.02);

  return (
    <div className="space-y-3 p-3 bg-black/20 rounded-lg border border-white/[0.04]">
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <label className="text-xs text-gray-400">
            {t('weapon.wear')}
            {activePreset && <span className="ml-1.5 text-amber-400/90 font-medium">{activePreset.label}</span>}
          </label>
          <input
            type="number"
            min={0}
            max={1}
            step={0.0001}
            value={wear}
            onChange={(e) => {
              const v = parseFloat(e.target.value);
              if (!Number.isNaN(v)) onWearChange(clampWear(v));
            }}
            className="num-input"
          />
        </div>
        <input
          type="range"
          min={0}
          max={1}
          step={0.001}
          value={wear}
          onChange={(e) => onWearChange(parseFloat(e.target.value))}
          className="w-full"
        />
        <div className="flex justify-between mt-1.5">
          {WEAR_PRESETS.map(p => (
            <button
              key={p.label}
              onClick={() => onWearChange(p.value)}
              className={`text-[10px] px-2 py-0.5 rounded-md transition-colors ${
                activePreset?.label === p.label
                  ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
                  : 'text-gray-500 hover:text-gray-300 border border-transparent'
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      <div>
        <div className="flex items-center justify-between mb-1.5">
          <label className="text-xs text-gray-400">
            {t('weapon.seed')} <span className="text-gray-600">({t('wearseed.randomHint')})</span>
          </label>
          <input
            type="number"
            min={0}
            max={1000}
            step={1}
            value={seed}
            onChange={(e) => {
              const v = parseInt(e.target.value, 10);
              onSeedChange(Number.isNaN(v) ? 0 : clampSeed(v));
            }}
            className="num-input"
          />
        </div>
        <input
          type="range"
          min={0}
          max={1000}
          step={1}
          value={seed}
          onChange={(e) => onSeedChange(parseInt(e.target.value, 10))}
          className="w-full"
        />
      </div>
    </div>
  );
}
