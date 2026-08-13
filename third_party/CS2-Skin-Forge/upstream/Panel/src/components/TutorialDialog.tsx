import { useState } from 'react';
import { useT, type I18nKey } from '../i18n';
import Modal from './ui/Modal';

interface TutorialDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const STEPS: { titleKey: I18nKey; descKey: I18nKey; icon: string }[] = [
  { titleKey: 'tutorial.step1.title', descKey: 'tutorial.step1.desc', icon: '📁' },
  { titleKey: 'tutorial.step2.title', descKey: 'tutorial.step2.desc', icon: '🚀' },
  { titleKey: 'tutorial.step3.title', descKey: 'tutorial.step3.desc', icon: '⚙️' },
  { titleKey: 'tutorial.step4.title', descKey: 'tutorial.step4.desc', icon: '🎨' },
  { titleKey: 'tutorial.step5.title', descKey: 'tutorial.step5.desc', icon: '🎮' },
];

export default function TutorialDialog({ isOpen, onClose }: TutorialDialogProps) {
  const { t } = useT();
  const [step, setStep] = useState(0);

  if (!isOpen) return null;

  const isLast = step === STEPS.length - 1;
  const current = STEPS[step];

  const handleClose = () => {
    setStep(0);
    onClose();
  };

  return (
    <Modal
      size="sm"
      title={t('tutorial.title')}
      onClose={handleClose}
      headerExtra={
        <button onClick={handleClose} className="btn-ghost !px-2 !py-1 text-xs">
          {t('tutorial.skip')}
        </button>
      }
      footer={
        <div className="flex gap-3">
          {step > 0 && (
            <button onClick={() => setStep(step - 1)} className="btn-secondary flex-1">
              {t('tutorial.prev')}
            </button>
          )}
          {isLast ? (
            <button onClick={handleClose} className="btn-primary flex-1">
              {t('tutorial.done')}
            </button>
          ) : (
            <button onClick={() => setStep(step + 1)} className="btn-primary flex-1">
              {t('tutorial.next')}
            </button>
          )}
        </div>
      }
    >
      <div className="space-y-5">
        {/* Progress dots */}
        <div className="flex items-center gap-1.5">
          {STEPS.map((_, i) => (
            <button
              key={i}
              onClick={() => setStep(i)}
              className={`h-1.5 rounded-full transition-all duration-200 ${
                i === step ? 'w-6 bg-amber-500' : i < step ? 'w-3 bg-amber-500/40' : 'w-3 bg-white/10'
              }`}
            />
          ))}
        </div>

        <div className="min-h-[150px] space-y-3">
          <div className="flex items-center gap-3">
            <div className="w-11 h-11 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-xl">
              {current.icon}
            </div>
            <div>
              <div className="text-[11px] font-medium text-gray-500">
                {step + 1} / {STEPS.length}
              </div>
              <h3 className="text-base font-semibold text-white">{t(current.titleKey)}</h3>
            </div>
          </div>
          <p className="text-sm text-gray-300 leading-relaxed">{t(current.descKey)}</p>
        </div>
      </div>
    </Modal>
  );
}
