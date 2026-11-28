import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  configTextArea: () => page.getByTestId('import-config-text'),
  importTextButton: () => page.getByTestId('import-config-text-button'),
  importFileButton: () => page.getByTestId('import-config-file-button'),
  subtitle: () => page.getByTestId('import-config-subtitle'),
});
