export const getOptions = (container: HTMLElement | null, selector: string) => {
  const options = container?.querySelectorAll<HTMLElement>(selector);

  if (options) {
    return Array.from(options);
  }

  return [];
};
