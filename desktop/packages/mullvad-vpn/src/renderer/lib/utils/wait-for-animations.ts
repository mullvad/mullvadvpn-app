export async function waitForAnimations(rootElement: HTMLElement | null | undefined) {
  if (rootElement && document.body.contains(rootElement)) {
    const animations = rootElement.getAnimations({ subtree: true });
    if (animations.length > 0) {
      await Promise.allSettled(animations.map((animation) => animation.finished));

      return new Promise((resolve) => {
        setTimeout(() => {
          resolve(waitForAnimations(rootElement));
        }, 0);
      });
    }
  }

  return Promise.resolve(null);
}
