import { useStore } from "../../state/store";
import { LANGUAGES } from "../../data/languages";

export default function LanguagesPage() {
  const { config, updateConfig } = useStore();
  const current = config?.language ?? null;

  return (
    <div className="lang-grid">
      {LANGUAGES.map((l) => (
        <button
          key={l.code}
          className={`lang-cell ${current === l.code ? "is-selected" : ""}`}
          onClick={() => updateConfig({ language: l.code })}
          title={l.code}
        >
          {l.native}
        </button>
      ))}
    </div>
  );
}
