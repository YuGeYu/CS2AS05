import { useT, type I18nKey } from "../i18n";

interface Tab {
  id: string;
  labelKey: I18nKey;
  icon: React.ReactNode;
}

const tabs: Tab[] = [
  { id: 'weapons', labelKey: 'tab.weapons', icon: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="3" strokeWidth="2" />
      <path strokeLinecap="round" strokeWidth="2" d="M12 2v4M12 18v4M2 12h4M18 12h4" />
    </svg>
  )},
  { id: 'knives', labelKey: 'tab.knives', icon: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M14 2l8 8-4 4-8-8 4-4zM6 12l-4 4 2 2 4-4-2-2z" />
    </svg>
  )},
  { id: 'gloves', labelKey: 'tab.gloves', icon: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M7 11V4a1 1 0 012 0v5m0-3a1 1 0 012 0v4m0-3a1 1 0 012 0v5m0-4a1 1 0 012 0v7a6 6 0 01-6 6H9a6 6 0 01-6-6v-4a2 2 0 014 0" />
    </svg>
  )},
  { id: 'agents', labelKey: 'tab.agents', icon: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
    </svg>
  )},
  { id: 'music', labelKey: 'tab.music', icon: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM21 16c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2z" />
    </svg>
  )},
];

interface TabNavigationProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
}

export default function TabNavigation({ activeTab, setActiveTab }: TabNavigationProps) {
  const { t } = useT();

  return (
    <nav className="flex gap-1 bg-black/30 p-1 rounded-xl border border-white/[0.06]">
      {tabs.map(tab => (
        <button
          key={tab.id}
          onClick={() => setActiveTab(tab.id)}
          className={`
            flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg
            transition-colors duration-150 text-sm font-medium
            ${activeTab === tab.id
              ? 'bg-amber-500 text-black shadow-lg shadow-amber-500/20'
              : 'text-gray-400 hover:text-white hover:bg-white/[0.06]'
            }
          `}
        >
          <span className="flex items-center">{tab.icon}</span>
          <span>{t(tab.labelKey)}</span>
        </button>
      ))}
    </nav>
  );
}
