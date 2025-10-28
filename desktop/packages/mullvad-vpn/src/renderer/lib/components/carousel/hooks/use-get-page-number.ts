import React from 'react';

import { useCarouselContext } from '../CarouselContext';

const PAGE_GAP = 16;

// Calculate the page number based on the scroll position.
export const useGetPageNumber = () => {
  const { content, pageContainerRef } = useCarouselContext();
  return React.useCallback(() => {
    if (pageContainerRef.current) {
      const scrollLeft = pageContainerRef.current.scrollLeft;
      const pageWidth = pageContainerRef.current.offsetWidth + PAGE_GAP;
      // Clamp it between 0 and props.content.length-1 to make sure it will correspond to a page.
      return Math.max(0, Math.min(Math.round(scrollLeft / pageWidth), content.length - 1));
    } else {
      return 0;
    }
  }, [content.length, pageContainerRef]);
};
