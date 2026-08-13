import type { Team } from '../utils/types';

interface TeamToggleProps {
  team: Team;
  onChange: (team: Team) => void;
  ctLabel: string;
  tLabel: string;
}

/** Segmented CT / T selector shared by weapon, knife, glove and agent panels. */
export default function TeamToggle({ team, onChange, ctLabel, tLabel }: TeamToggleProps) {
  return (
    <div className="flex p-1 gap-1 bg-black/30 rounded-lg border border-white/[0.06]">
      <button
        onClick={() => onChange('ct')}
        className={`flex-1 py-1.5 rounded-md text-sm font-semibold transition-colors duration-150 ${
          team === 'ct'
            ? 'bg-sky-500/20 text-sky-300 border border-sky-500/40'
            : 'text-gray-400 hover:text-gray-200 border border-transparent'
        }`}
      >
        {ctLabel}
      </button>
      <button
        onClick={() => onChange('t')}
        className={`flex-1 py-1.5 rounded-md text-sm font-semibold transition-colors duration-150 ${
          team === 't'
            ? 'bg-orange-500/20 text-orange-300 border border-orange-500/40'
            : 'text-gray-400 hover:text-gray-200 border border-transparent'
        }`}
      >
        {tLabel}
      </button>
    </div>
  );
}
