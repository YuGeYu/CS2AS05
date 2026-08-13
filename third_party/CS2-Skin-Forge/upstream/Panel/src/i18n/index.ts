import { useCallback, useSyncExternalStore } from "react";
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

const LANGUAGE_KEY = "cs2skinmod.language";

/** Detect system language from navigator / OS. */
function detectSystemLanguage(): string {
  try {
    // navigator.language gives e.g. "zh-CN", "ja", "ko-KR", "en-US"
    const raw = (navigator.language || (navigator as any).userLanguage || "").toLowerCase();
    if (raw.startsWith("zh")) {
      if (raw.includes("tw") || raw.includes("hk") || raw.includes("mo") || raw === "zh-hant") {
        return "tchinese";
      }
      return "schinese";
    }
    if (raw.startsWith("ja")) return "japanese";
    if (raw.startsWith("ko")) return "koreana";
    if (raw.startsWith("ru")) return "russian";
    return "english";
  } catch {
    return "english";
  }
}

/** Get the saved language preference, falling back to system detection. */
export function getSavedLanguage(): string {
  try {
    return localStorage.getItem(LANGUAGE_KEY) ?? detectSystemLanguage();
  } catch {
    return detectSystemLanguage();
  }
}

/** Save language preference */
export function saveLanguage(lang: string): void {
  try {
    localStorage.setItem(LANGUAGE_KEY, lang);
  } catch {
    /* localStorage unavailable */
  }
}

/** Available languages */
export const LANGUAGES = [
  { code: "english", label: "English", flag: "EN" },
  { code: "schinese", label: "简体中文", flag: "CN" },
  { code: "tchinese", label: "繁體中文", flag: "TW" },
  { code: "japanese", label: "日本語", flag: "JP" },
  { code: "koreana", label: "한국어", flag: "KR" },
  { code: "russian", label: "Русский", flag: "RU" },
] as const;

/** Hook returning a translator bound to the current language.
 *
 * Language state lives in a module-level store (not per-component state) so
 * that changing the language re-renders every component that calls useT().
 */
let currentLang = getSavedLanguage();
const listeners = new Set<() => void>();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Change the app language globally and persist the preference. */
export function setLanguage(lang: string): void {
  if (lang === currentLang) return;
  currentLang = lang;
  saveLanguage(lang);
  listeners.forEach((l) => l());
}

export function useT() {
  const lang = useSyncExternalStore(subscribe, () => currentLang);

  const t = useCallback(
    (key: I18nKey, params?: TParams) => translate(lang, key, params),
    [lang]
  );

  return { t, lang, changeLanguage: setLanguage, languages: LANGUAGES };
}
