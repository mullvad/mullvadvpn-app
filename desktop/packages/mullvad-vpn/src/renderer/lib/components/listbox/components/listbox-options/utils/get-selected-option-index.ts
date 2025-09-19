import { getIsOptionSelected } from './get-is-option-selected';

export const getSelectedOptionIndex = (options?: NodeListOf<HTMLElement>) => {
  if (!options) return -1;

  return Array.from(options).findIndex((option) => getIsOptionSelected(option));
};
