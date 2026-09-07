import { useEffect, useState } from "react";
import Section from "../components/Section";
import type { Status } from "../components/StatusDot";
import { useStore } from "../state/store";
import { useT } from "../i18n";
import { KNIFE_ICONS } from "../data/knifeIcons";
import { captureKeyName } from "../lib/keycapture";
import "./DropKnivesSection.css";

export default function DropKnivesSection() {
  const { dropKnives, csgoPath, applyDropKnives, dropKnivesPending } = useStore();
  const t = useT();
  const [capturing, setCapturing] = useState(false);

  const bindKey = dropKnives?.bind_key ?? "\\";
  const selected = new Set(dropKnives?.selected ?? []);
  const cfgPresent = dropKnives?.cfg_present ?? false;
  const running = dropKnives?.cs2_running ?? false;
  const disabled = !csgoPath || !cfgPresent;

  // Yellow only if Drop Knives was changed while CS2 is running (pending restart).
  const status: Status =
    !csgoPath ? "off" : !cfgPresent ? "red" : running && dropKnivesPending ? "yellow" : "green";

  // Key capture: grab the first keydown after the box is clicked.
  useEffect(() => {
    if (!capturing) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const name = captureKeyName(e);
      setCapturing(false);
      applyDropKnives(name, Array.from(selected));
    };
    window.addEventListener("keydown", onKey, { capture: true, once: true });
    return () => window.removeEventListener("keydown", onKey, { capture: true } as any);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [capturing]);

  const toggle = (id: number) => {
    if (disabled) return;
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    // keep numeric order for a stable bind string
    const ordered = KNIFE_ICONS.map((k) => k.id).filter((i) => next.has(i));
    applyDropKnives(bindKey, ordered);
  };

  return (
    <Section title={t("pre.dropKnives")} status={status}>
      <div className="dk__bind">
        <span className="dk__bind-label">{t("pre.bind")}</span>
        <button
          className={`dk__bind-box ${capturing ? "is-capturing" : ""}`}
          disabled={disabled}
          onClick={() => setCapturing(true)}
          title="Click, then press a key"
        >
          {capturing ? t("pre.pressKey") : bindKey === "\\" ? "\\" : bindKey}
        </button>
      </div>

      <div className={`dk__grid ${disabled ? "is-disabled" : ""}`}>
        {KNIFE_ICONS.map((k) => (
          <button
            key={k.id}
            className={`dk__knife ${selected.has(k.id) ? "is-selected" : ""}`}
            onClick={() => toggle(k.id)}
            disabled={disabled}
            title={`subclass_create ${k.id}`}
            aria-pressed={selected.has(k.id)}
          >
            <img src={k.url} alt={`knife ${k.id}`} draggable={false} />
          </button>
        ))}
      </div>
    </Section>
  );
}
