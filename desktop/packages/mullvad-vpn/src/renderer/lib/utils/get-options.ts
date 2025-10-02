export const getOptions = (container: HTMLElement | null, selector?: string) => {
  const querySelector = selector ?? '[data-option="true"]';
  const options = container?.querySelectorAll<HTMLElement>(
    `${querySelector}:not([aria-disabled="true"])`,
  );

  if (options) {
    return Array.from(options);
  }

  return [];
};
