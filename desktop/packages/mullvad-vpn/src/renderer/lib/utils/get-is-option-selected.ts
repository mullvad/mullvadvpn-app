export const getIsOptionSelected = (option: HTMLElement) => {
  return option.getAttribute('aria-selected') === 'true';
};
