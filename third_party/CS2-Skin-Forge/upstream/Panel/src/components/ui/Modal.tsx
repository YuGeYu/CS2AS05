import { useEffect, useCallback } from 'react';

type ModalSize = 'sm' | 'md' | 'lg' | 'editor';

interface ModalProps {
  title?: React.ReactNode;
  onClose: () => void;
  children: React.ReactNode;
  /** sm = compact dialog, md = settings-style, lg = wide, editor = near-fullscreen item editor */
  size?: ModalSize;
  /** Extra content rendered on the right side of the title bar (before the ✕ button). */
  headerExtra?: React.ReactNode;
  /** Sticky footer content (action buttons etc). */
  footer?: React.ReactNode;
  /** Set false to hide the ✕ button and disable ESC/backdrop dismissal. */
  dismissable?: boolean;
}

const SIZE_CLASS: Record<ModalSize, string> = {
  sm: 'max-w-md',
  md: 'max-w-lg',
  lg: 'max-w-3xl',
  editor: 'max-w-6xl w-[96vw] h-[90vh]',
};

/**
 * Shared modal shell: backdrop, ESC-to-close, click-outside-to-close,
 * title bar and optional sticky footer. All app dialogs build on this so
 * behavior and styling stay consistent.
 */
export default function Modal({
  title,
  onClose,
  children,
  size = 'md',
  headerExtra,
  footer,
  dismissable = true,
}: ModalProps) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape' && dismissable) onClose();
    },
    [onClose, dismissable]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget && dismissable) onClose();
      }}
    >
      <div className={`card w-full ${SIZE_CLASS[size]} max-h-[92vh] flex flex-col !p-0 overflow-hidden`}>
        {/* Title bar */}
        {(title !== undefined || dismissable) && (
          <div className="flex items-center justify-between gap-3 px-5 py-3.5 border-b border-white/[0.06] shrink-0">
            <h2 className="text-base font-bold text-white truncate flex items-center gap-2">{title}</h2>
            <div className="flex items-center gap-2 shrink-0">
              {headerExtra}
              {dismissable && (
                <button
                  onClick={onClose}
                  className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/[0.07] transition-colors"
                  aria-label="Close"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              )}
            </div>
          </div>
        )}

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-5 py-4">{children}</div>

        {/* Footer */}
        {footer && (
          <div className="px-5 py-3 border-t border-white/[0.06] shrink-0 bg-black/20">{footer}</div>
        )}
      </div>
    </div>
  );
}
