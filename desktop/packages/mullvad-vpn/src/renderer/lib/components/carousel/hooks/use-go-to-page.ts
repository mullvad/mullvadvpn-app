import React from 'react';

import { useCarouselContext } from '../CarouselContext';

// Scroll to a specific page.
export function useGoToPage() {
  const { pageContainerRef } = useCarouselContext();
  return React.useCallback(
    (page: number) => {
      if (pageContainerRef.current) {
        const width = pageContainerRef.current.offsetWidth;
        pageContainerRef.current.scrollTo({ left: width * page });
      }
    },
    [pageContainerRef],
  );
}
