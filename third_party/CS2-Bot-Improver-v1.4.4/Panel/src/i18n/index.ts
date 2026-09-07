import { useStore } from "../state/store";
import { EN, type I18nKey } from "./keys";
import { DICTS } from "./dictionary";

export type { I18nKey };

export type TParams = Record<string, string | number>;

/** Translate a key for a locale, falling back to English, then the key itself. */
export function translate(lang: string | null, key: I18nKey, params?: TParams): string {
  const dict = lang ? DICTS[lang] : undefined;
  let s: string = (dict && dict[key]) ?? EN[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.replace(`{${k}}`, String(v));
    }
  }
  return s;
}

/** Hook returning a translator bound to the current language. */
export function useT() {
  const { config } = useStore();
  const lang = config?.language ?? "english";
  return (key: I18nKey, params?: TParams) => translate(lang, key, params);
}
