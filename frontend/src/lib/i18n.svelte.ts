import en from './locales/en';
import tr from './locales/tr';

export type Locale = 'en' | 'tr';

function getInitialLocale(): Locale {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem('gwl_locale');
    if (saved === 'tr' || saved === 'en') return saved;
  }
  if (typeof navigator !== 'undefined') {
    const lang = (navigator.language || '').toLowerCase();
    if (lang.startsWith('tr')) return 'tr';
  }
  return 'en';
}

class I18nStore {
  locale = $state<Locale>(getInitialLocale());

  setLocale(l: Locale) {
    this.locale = l;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('gwl_locale', l);
    }
  }

  t(key: string, params?: Record<string, string | number>): string {
    const dict = this.locale === 'tr' ? tr : en;
    let text = dict[key] ?? en[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
      }
    }
    return text;
  }
}

export const i18n = new I18nStore();
