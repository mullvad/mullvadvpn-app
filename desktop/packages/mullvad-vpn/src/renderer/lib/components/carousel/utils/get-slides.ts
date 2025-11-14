export const getSlides = (container: HTMLElement | null) => {
  const slides = container?.querySelectorAll<HTMLElement>('[data-carousel-slide]');
  if (slides) {
    return Array.from(slides);
  }

  return [];
};
