import { useState } from "react";
import Card from "../components/Card";
import Segmented from "../components/Segmented";
import { useToast } from "../components/Toast";
import { useStore } from "../state/store";
import { useT } from "../i18n";
import type { DifficultyLevel } from "../lib/api";

const LEVELS: { value: DifficultyLevel; label: string }[] = [
  { value: "Low", label: "Low" },
  { value: "Medium", label: "Medium" },
  { value: "High", label: "High" },
];

export default function DifficultyCard() {
  const { difficulty, config, csgoPath, applyDifficulty, difficultyPending } = useStore();
  const toast = useToast();
  const t = useT();
  const [pending, setPending] = useState<DifficultyLevel | null>(null);

  // Optimistic: show the clicked level immediately, revert if the swap fails.
  // On-disk detection wins; fall back to remembered, then default Medium.
  const current: DifficultyLevel =
    pending ??
    difficulty?.current ??
    ((config?.difficulty as DifficultyLevel | null) ?? "Medium");

  // Yellow only if the difficulty was changed while CS2 is running.
  const tone = difficulty?.cs2_running && difficultyPending ? "yellow" : "green";

  const onChange = async (level: DifficultyLevel) => {
    setPending(level);
    const info = await applyDifficulty(level);
    setPending(null);
    if (!info) return;
    if (info.cs2_running) {
      toast.show(t("common.restart"), "neutral");
    } else {
      toast.show(`${t("diff.title")}: ${level}`, "green");
    }
  };

  return (
    <Card title={t("diff.title")}>
      <Segmented
        ariaLabel="Difficulty"
        value={current}
        onChange={onChange}
        disabled={!csgoPath}
        options={LEVELS.map((l) => ({
          ...l,
          tone:
            l.value === current
              ? (tone as "green" | "yellow")
              : undefined,
        }))}
      />
    </Card>
  );
}
