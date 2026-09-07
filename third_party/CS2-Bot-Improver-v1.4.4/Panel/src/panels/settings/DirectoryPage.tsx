import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "../../state/store";
import { useT } from "../../i18n";
import StatusDot from "../../components/StatusDot";

export default function DirectoryPage() {
  const { directory, chooseDirectory, reportError } = useStore();
  const t = useT();
  const candidates = directory?.candidates ?? [];
  const selected = directory?.selected ?? null;

  const browse = async () => {
    try {
      const picked = await open({ directory: true, title: "Select game/csgo folder" });
      if (typeof picked === "string") await chooseDirectory(picked);
    } catch (e) {
      reportError(e);
    }
  };

  return (
    <div className="settings-list">
      {!directory?.steam_found && (
        <div className="dir-note">{t("set.steamNotDetected")}</div>
      )}

      {candidates.map((path) => {
        const isSel = path === selected;
        return (
          <button
            key={path}
            className={`dir-cell ${isSel ? "is-selected" : ""}`}
            onClick={() => chooseDirectory(path)}
          >
            <span className="dir-cell__path">{path}</span>
            {isSel && <StatusDot status="green" />}
          </button>
        );
      })}

      {candidates.length === 0 && (
        <div className="dir-note">{t("set.noCsgo")}</div>
      )}

      <button className="settings-row settings-row--action" onClick={browse}>
        {t("set.browse")}
      </button>
    </div>
  );
}
