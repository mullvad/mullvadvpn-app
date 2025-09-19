export const getOptions = (container: HTMLElement | null) => {
  if (!container) return;
  return container?.querySelectorAll('[role="option"]') as NodeListOf<HTMLElement>;
};
