import { useState, useEffect } from 'react';
import { useT } from '../i18n';
import { openUrl } from "@tauri-apps/plugin-opener";

const DISCLAIMER_KEY = 'cs2skinmod.disclaimer.accepted';

interface DisclaimerDialogProps {
  onAccepted?: () => void;
}

export default function DisclaimerDialog({ onAccepted }: DisclaimerDialogProps) {
  const { lang } = useT();
  const [show, setShow] = useState(false);
  const [accepted, setAccepted] = useState(false);
  const isChinese = lang === 'schinese' || lang === 'tchinese';

  useEffect(() => {
    try {
      const dismissed = localStorage.getItem(DISCLAIMER_KEY);
      if (!dismissed) setShow(true);
    } catch { setShow(true); }
  }, []);

  const handleConfirm = () => {
    try { localStorage.setItem(DISCLAIMER_KEY, 'true'); } catch {}
    setShow(false);
    onAccepted?.();
  };

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="card w-full max-w-lg mx-4 space-y-4">
        <div className="text-center">
          <div className="text-4xl mb-2">⚠️</div>
          <h2 className="text-xl font-bold text-white">
            {isChinese ? '使用须知' : 'Important Notice'}
          </h2>
        </div>

        <div className="space-y-3 text-sm text-gray-300 leading-relaxed">
          <p>
            {isChinese
              ? '这是一个开源免费软件 (GPL-3.0)，仅建议在本地离线游戏中使用。'
              : 'This is open-source free software (GPL-3.0), recommended for local/offline use only.'}
          </p>
          <p className="text-amber-400 font-medium">
            {isChinese
              ? '⚠️ 使用第三方软件修改 CS2 存在 VAC 封禁风险。强烈建议使用小号体验！'
              : '⚠️ Using third-party software with CS2 carries VAC ban risk. Strongly recommended to use an alt account!'}
          </p>
          <p>
            {isChinese
              ? '请勿上当受骗，本软件完全免费开源。'
              : 'This software is completely free and open source. Do not pay for it.'}
          </p>
          <span
            onClick={() => openUrl("https://github.com/kaecho/CS2-Skin-Forge")}
            className="text-blue-400 hover:text-blue-300 underline text-xs block cursor-pointer"
          >
            github.com/kaecho/CS2-Skin-Forge
          </span>
        </div>

        <label className="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            checked={accepted}
            onChange={(e) => setAccepted(e.target.checked)}
            className="w-4 h-4 rounded accent-amber-500"
          />
          <span className="text-sm text-gray-300">
            {isChinese ? '我已了解以上信息' : 'I understand the above'}
          </span>
        </label>

        <button
          onClick={handleConfirm}
          disabled={!accepted}
          className={`w-full py-2.5 rounded-lg text-sm font-semibold transition-all ${
            accepted
              ? 'bg-amber-600 hover:bg-amber-500 text-white'
              : 'bg-gray-700 text-gray-500 cursor-not-allowed'
          }`}
        >
          {isChinese ? '确认' : 'Confirm'}
        </button>
      </div>
    </div>
  );
}
