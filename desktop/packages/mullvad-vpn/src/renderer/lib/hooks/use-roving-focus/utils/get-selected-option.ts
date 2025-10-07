import { getIsOptionSelected } from './get-is-option-selected';

export const getSelectedOption = (options: HTMLElement[]) => {
  return options.find((option) => getIsOptionSelected(option));
};
