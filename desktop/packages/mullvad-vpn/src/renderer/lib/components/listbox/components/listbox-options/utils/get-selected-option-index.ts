import { getIsOptionSelected } from './get-is-option-selected';

export const getSelectedOptionIndex = (options: HTMLElement[]) => {
  return options.findIndex((option) => getIsOptionSelected(option));
};
