export function focusFirstFocusableHeading() {
  const firstFocusableHeading = document.querySelector<HTMLElement>('[data-focusable-heading]');
  firstFocusableHeading?.focus();
}
