export const getOptions = (container: HTMLElement | null) => {
  const options = container?.querySelectorAll<HTMLElement>(
    '[role="option"]:not([aria-disabled="true"])',
  );

  if (options) {
    return Array.from(options);
  }

  return [];
};
