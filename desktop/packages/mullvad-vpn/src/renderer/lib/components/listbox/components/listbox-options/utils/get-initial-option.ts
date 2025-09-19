import { getSelectedOption } from './get-selected-option';

export const getInitialOption = (options?: NodeListOf<HTMLElement>): HTMLElement | undefined => {
  const selectedOption = getSelectedOption(options);
  if (selectedOption) {
    return selectedOption;
  }

  return options?.length ? options[0] : undefined;
};
