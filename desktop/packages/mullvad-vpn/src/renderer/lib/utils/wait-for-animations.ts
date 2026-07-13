export async function waitForAnimations(rootElement: HTMLElement | null | undefined) {
  if (!rootElement || !document.body.contains(rootElement)) {
    return Promise.resolve();
  }
  const animations = rootElement.getAnimations({ subtree: true });
  await Promise.allSettled(animations.map((animation) => animation.finished));
}
