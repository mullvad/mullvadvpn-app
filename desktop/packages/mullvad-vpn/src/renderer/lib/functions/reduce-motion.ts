export function getReduceMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
