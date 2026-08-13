import { useT } from '../i18n';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { UpdateCheckResult } from '../lib/api';

interface UpdateBannerProps {
  update: UpdateCheckResult | null;
  onDismiss: () => void;
}

export default function UpdateBanner({ update, onDismiss }: UpdateBannerProps) {
  const { t } = useT();

  if (!update || !update.updateAvailable) return null;

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-amber-500/10 border-b border-amber-500/20">
      <svg className="w-4 h-4 text-amber-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
      </svg>
      <span className="text-xs text-amber-200 flex-1 truncate">
        {t('update.available', { version: update.latestVersion })}
      </span>
      <button
        onClick={() => openUrl(update.releaseUrl)}
        className="text-xs font-semibold text-black bg-amber-500 hover:bg-amber-400 px-3 py-1 rounded-md transition-colors"
      >
        {t('update.download')}
      </button>
      <button onClick={onDismiss} className="text-xs text-gray-400 hover:text-white transition-colors px-1">
        ✕
      </button>
    </div>
  );
}
