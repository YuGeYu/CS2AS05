import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalIcon } from "../../components/icons";
import { useStore } from "../../state/store";
import { useT } from "../../i18n";
import { DEVS, PROJECT_URL } from "../../data/devs";

export default function DevsPage() {
  const { reportError } = useStore();
  const t = useT();

  const openProject = async () => {
    try {
      await openUrl(PROJECT_URL);
    } catch (e) {
      reportError(e);
    }
  };

  return (
    <div className="settings-list">
      <button
        className="settings-row settings-row--link settings-row--project"
        onClick={openProject}
      >
        <div className="settings-row__text">
          <span className="settings-row__title">{t("set.project")}</span>
          <span className="settings-row__sub">{PROJECT_URL}</span>
        </div>
        <ExternalIcon size={18} />
      </button>

      <div className="devs">
        {DEVS.map((name) => (
          <div className="devs__item" key={name}>
            {name}
          </div>
        ))}
      </div>
    </div>
  );
}
