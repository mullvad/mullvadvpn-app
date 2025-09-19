import { getIsOptionSelected } from './get-is-option-selected';

export const getSelectedOption = (options?: NodeListOf<HTMLElement>): HTMLElement | undefined => {
  if (!options) return undefined;

  return Array.from(options).find((option) => getIsOptionSelected(option));
};
