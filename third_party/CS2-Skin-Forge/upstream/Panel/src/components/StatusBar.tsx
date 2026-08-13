interface StatusBarProps {
  status: { message: string; type: 'success' | 'error' | 'info' } | null;
}

export default function StatusBar({ status }: StatusBarProps) {
  if (!status) return null;

  const styles = {
    success: 'border-emerald-500/40 bg-emerald-500/[0.12] text-emerald-200',
    error: 'border-red-500/40 bg-red-500/[0.12] text-red-200',
    info: 'border-sky-500/40 bg-sky-500/[0.12] text-sky-200',
  }[status.type];

  const icon = {
    success: 'M5 13l4 4L19 7',
    error: 'M6 18L18 6M6 6l12 12',
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  }[status.type];

  return (
    <div className="fixed bottom-5 left-1/2 -translate-x-1/2 z-50 pointer-events-none">
      <div className={`flex items-center gap-2 px-4 py-2.5 rounded-xl border backdrop-blur-md shadow-xl ${styles}`}>
        <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={icon} />
        </svg>
        <span className="text-xs font-medium">{status.message}</span>
      </div>
    </div>
  );
}
