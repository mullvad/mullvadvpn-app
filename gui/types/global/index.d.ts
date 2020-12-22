import Gettext from 'node-gettext';

declare global {
  interface Window {
    loadTranslations(locale: string, catalogue: Gettext): void;
  }
}
