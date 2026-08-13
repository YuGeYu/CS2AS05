import { useState } from 'react';
import { Loadout, Team } from '../utils/types';
import { agentModels } from '../data/skins';
import { getLocalizedName, agentNameMap } from '../data/localNames';
import { useT } from '../i18n';
import TeamToggle from './TeamToggle';

interface AgentPanelProps {
  loadout: Loadout;
  updateLoadout: (updates: Partial<Loadout>) => void;
}

export default function AgentPanel({ loadout, updateLoadout }: AgentPanelProps) {
  const { t, lang } = useT();
  const [selectedTeam, setSelectedTeam] = useState<Team>('ct');

  const handleAgentSelect = (modelId: string) => {
    const arrayIndex = models.findIndex(m => m.id === modelId);
    if (arrayIndex < 0) return;
    const selectedModel = models[arrayIndex];
    if (selectedTeam === 'ct') {
      updateLoadout({ agentModelCt: arrayIndex, agentModelPathCt: selectedModel.model, useRandom: false });
    } else {
      updateLoadout({ agentModelT: arrayIndex, agentModelPathT: selectedModel.model, useRandom: false });
    }
  };

  const handleRandom = () => {
    updateLoadout({ agentModelCt: -1, agentModelT: -1, agentModelPathCt: '', agentModelPathT: '', useRandom: true });
  };

  const models = selectedTeam === 'ct' ? agentModels.ct : agentModels.t;
  const currentIdx = selectedTeam === 'ct' ? loadout.agentModelCt : loadout.agentModelT;
  const isBothRandom = loadout.agentModelCt === -1 && loadout.agentModelT === -1;

  // Find selected agent names for display
  const selectedCtIdx = loadout.agentModelCt >= 0
    ? Math.min(loadout.agentModelCt, agentModels.ct.length - 1)
    : -1;
  const selectedCtName = selectedCtIdx >= 0
    ? getLocalizedName('agent-' + agentModels.ct[selectedCtIdx].id, agentNameMap, lang, agentModels.ct[selectedCtIdx].name)
    : t("preview.random");
  const selectedTIdx = loadout.agentModelT >= 0
    ? Math.min(loadout.agentModelT, agentModels.t.length - 1)
    : -1;
  const selectedTName = selectedTIdx >= 0
    ? getLocalizedName('agent-' + agentModels.t[selectedTIdx].id, agentNameMap, lang, agentModels.t[selectedTIdx].name)
    : t("preview.random");

  return (
    <div className="space-y-3">
      <TeamToggle team={selectedTeam} onChange={setSelectedTeam} ctLabel={t("agent.ct")} tLabel={t("agent.t")} />

      {/* Show current selections for both teams */}
      <div className="flex gap-2 text-xs">
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-sky-400 font-semibold">{t("agent.ct")}:</span>{' '}
          <span className="text-gray-200">{selectedCtName}</span>
        </div>
        <div className="flex-1 rounded-lg bg-black/20 border border-white/[0.06] px-3 py-2">
          <span className="text-orange-400 font-semibold">{t("agent.t")}:</span>{' '}
          <span className="text-gray-200">{selectedTName}</span>
        </div>
      </div>

      <button
        onClick={handleRandom}
        className={`card card-hover w-full text-center py-3 ${isBothRandom ? 'card-selected' : ''}`}
      >
        <div className="text-sm font-semibold text-white">{t("preview.random")}</div>
        <div className="text-xs text-gray-400 mt-0.5">{t("agent.title")}</div>
      </button>

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        {models.map((model, index) => (
          <button
            key={model.id}
            onClick={() => handleAgentSelect(model.id)}
            className={`card card-hover !p-2.5 text-center ${currentIdx === index ? 'card-selected' : ''}`}
          >
            {model.image ? (
              <img src={model.image} alt={model.name}
                className="w-full h-16 object-contain mb-1"
                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
            ) : (
              <div className={`w-10 h-10 mx-auto mb-1 rounded-full flex items-center justify-center text-white text-xs font-bold ${
                selectedTeam === 'ct' ? 'bg-sky-600/50' : 'bg-orange-600/50'
              }`}>
                {model.name.charAt(0)}
              </div>
            )}
            <div className="text-xs font-medium text-white truncate">{getLocalizedName('agent-' + model.id, agentNameMap, lang, model.name)}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
