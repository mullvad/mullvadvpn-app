import React from 'react';

import { useCarouselContext } from '../CarouselContext';
import { useGetPageNumber } from './use-get-page-number';
import { useGoToPage } from './use-go-to-page';

export function usePages() {
  const { content, pageNumber } = useCarouselContext();
  const goToPage = useGoToPage();
  const getPageNumber = useGetPageNumber();

  // These values are only intended to be used for display purposes. Using them when calculating
  // next or prev page would increase the risk of race conditions.
  const hasNext = pageNumber < content.length - 1;
  const hasPrev = pageNumber > 0;

  const next = React.useCallback(() => goToPage(getPageNumber() + 1), [goToPage, getPageNumber]);
  const prev = React.useCallback(() => goToPage(getPageNumber() - 1), [goToPage, getPageNumber]);

  return {
    goToPage,
    next,
    prev,
    hasNext,
    hasPrev,
  };
}
