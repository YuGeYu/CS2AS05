import { useStore } from "../../state/store";
import { LANGUAGES } from "../../data/languages";
import "./settings.css";

// Shown on first launch (config.first_run_done === false). Picking a language
// also marks first-run complete.
export default function FirstRunLanguages() {
  const { updateConfig } = useStore();

  return (
    <div className="firstrun">
      <div className="firstrun__card glass glass-strong">
        <h2 className="firstrun__title">Language</h2>
        <div className="lang-grid">
          {LANGUAGES.map((l) => (
            <button
              key={l.code}
              className="lang-cell"
              onClick={() => updateConfig({ language: l.code, first_run_done: true })}
            >
              {l.native}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
