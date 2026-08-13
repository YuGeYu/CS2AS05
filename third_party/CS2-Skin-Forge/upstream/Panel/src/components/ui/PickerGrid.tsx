import { useState, useMemo, useEffect } from 'react';
import { useT } from '../../i18n';

export interface PickerItem {
  id: number;
  name: string;
  image?: string;
}

interface PickerGridProps {
  items: PickerItem[];
  selectedId: number | null;
  onSelect: (id: number) => void;
  /** When provided, a "Default / none" tile is rendered first and clicking it calls onClear. */
  onClear?: () => void;
  clearLabel?: string;
  searchPlaceholder?: string;
  /** Tile size: lg for skins (big preview), sm for stickers/keychains (dense grid). */
  tile?: 'lg' | 'sm';
  pageSize?: number;
  emptyText?: string;
}

/**
 * Searchable, incrementally-loaded item grid used by every picker
 * (weapon skins, knife skins, glove paints, stickers, keychains).
 */
export default function PickerGrid({
  items,
  selectedId,
  onSelect,
  onClear,
  clearLabel,
  searchPlaceholder,
  tile = 'lg',
  pageSize = 150,
  emptyText,
}: PickerGridProps) {
  const { t } = useT();
  const [search, setSearch] = useState('');
  const [limit, setLimit] = useState(pageSize);

  // Reset pagination when the query or the item SET changes. Depend on
  // items.length (not the array reference) — parents rebuild the array via
  // .map() on every render, and resetting then would snap "Load More"
  // pagination back to page one after each selection.
  useEffect(() => {
    setLimit(pageSize);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [search, items.length, pageSize]);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return items;
    return items.filter((item) => item.name.toLowerCase().includes(q));
  }, [items, search]);

  const visible = useMemo(() => filtered.slice(0, limit), [filtered, limit]);

  const gridClass =
    tile === 'lg'
      ? 'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 gap-2.5'
      : 'grid grid-cols-3 sm:grid-cols-5 md:grid-cols-7 xl:grid-cols-8 gap-2';
  const imgClass = tile === 'lg' ? 'w-full h-20 object-contain mb-1.5' : 'w-12 h-12 object-contain mb-1';

  return (
    <div className="space-y-3">
      <input
        type="text"
        placeholder={searchPlaceholder ?? t('picker.searchSkins')}
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        className="input-field !text-sm"
        autoFocus
      />

      <div className={gridClass}>
        {onClear && !search.trim() && (
          <button
            onClick={onClear}
            className={`flex flex-col items-center justify-center p-2.5 rounded-lg text-xs font-medium transition-all duration-150 border ${
              selectedId === null
                ? 'bg-amber-500/[0.15] text-amber-200 border-amber-500/50'
                : 'bg-white/[0.03] text-gray-400 hover:bg-white/[0.07] border-dashed border-white/[0.12]'
            }`}
          >
            <svg className={tile === 'lg' ? 'w-10 h-10 mb-1.5 opacity-50' : 'w-6 h-6 mb-1 opacity-50'} fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
            <span className="truncate w-full text-center leading-tight">{clearLabel ?? t('picker.default')}</span>
          </button>
        )}
        {visible.map((item) => {
          const isSelected = item.id === selectedId;
          return (
            <button
              key={item.id}
              onClick={() => onSelect(item.id)}
              title={item.name}
              className={`flex flex-col items-center p-2.5 rounded-lg text-xs font-medium transition-all duration-150 border ${
                isSelected
                  ? 'bg-amber-500/[0.15] text-amber-200 border-amber-500/50 shadow-[0_0_12px_rgba(245,158,11,0.15)]'
                  : 'bg-white/[0.04] text-gray-300 hover:bg-white/[0.08] border-white/[0.05] hover:border-white/[0.12]'
              }`}
            >
              {item.image && (
                <img
                  src={item.image}
                  alt={item.name}
                  className={imgClass}
                  loading="lazy"
                  onError={(e) => {
                    (e.target as HTMLImageElement).style.display = 'none';
                  }}
                />
              )}
              <span className="truncate w-full text-center leading-tight">{item.name}</span>
            </button>
          );
        })}
        {visible.length === 0 && (
          <div className="col-span-full text-center text-sm text-gray-600 py-10">
            {emptyText ?? t('picker.noResults')}
          </div>
        )}
      </div>

      {filtered.length > limit && (
        <button
          onClick={() => setLimit((prev) => prev + pageSize)}
          className="w-full py-2 text-sm text-gray-300 bg-white/[0.05] hover:bg-white/10 rounded-lg border border-white/[0.06] transition-colors"
        >
          {t('common.loadMore', { shown: visible.length, total: filtered.length })}
        </button>
      )}
    </div>
  );
}
