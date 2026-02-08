import React from 'react';

import { Accordion } from '../../../../lib/components/accordion';
import { useAccordionContext } from '../../../../lib/components/accordion/AccordionContext';
import type { AccordionContentProps } from '../../../../lib/components/accordion/components';
import { useScrollPositionContext } from '../../../views/select-location/ScrollPositionContext';

export type LocationAccordionProps = AccordionContentProps;

export const LocationAccordionContent = (props: LocationAccordionProps) => {
  const { headerRef, content } = useAccordionContext();
  const { spacePreAllocationViewRef, scrollIntoView, resetHeight, scrollViewRef } =
    useScrollPositionContext();

  React.useLayoutEffect(() => {
    if (!content) return;

    const viewportBottom =
      scrollViewRef.current?.scrollableRef.current?.offsetHeight ?? window.innerHeight;
    const { scrollHeight } = content;

    const rect = content.getBoundingClientRect();

    const predictedBottom = rect.top + scrollHeight;
    const predictedRect = new DOMRect(rect.left, rect.top, rect.width, scrollHeight);

    const contentIsLargerThanViewport = scrollHeight > viewportBottom;
    if (contentIsLargerThanViewport) {
      spacePreAllocationViewRef.current?.allocate(viewportBottom + predictedRect.height);
      headerRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
      resetHeight();
      return;
    }

    const contentIsBelowViewport = predictedBottom > viewportBottom;
    if (contentIsBelowViewport) {
      const isBelow = predictedRect.bottom > viewportBottom;
      if (isBelow) {
        spacePreAllocationViewRef.current?.allocate(viewportBottom + predictedRect.height);
        scrollIntoView(predictedRect);
        resetHeight();
      }
    }
  }, [content, headerRef, resetHeight, scrollIntoView, scrollViewRef, spacePreAllocationViewRef]);
  return <Accordion.Content {...props} />;
};
