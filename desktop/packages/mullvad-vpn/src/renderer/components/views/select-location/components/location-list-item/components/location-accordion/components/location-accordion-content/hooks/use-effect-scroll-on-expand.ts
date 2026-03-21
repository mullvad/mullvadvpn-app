import React from 'react';

import { useAccordionContext } from '../../../../../../../../../../lib/components/accordion/AccordionContext';
import { useScrollPositionContext } from '../../../../../../../ScrollPositionContext';
import { useLocationAccordionContext } from '../../../LocationAccordionContext';

export function useEffectScrollOnExpand() {
  const { userTriggeredExpand, setUserTriggeredExpand } = useLocationAccordionContext();
  const { headerRef, content } = useAccordionContext();
  const { spacePreAllocationViewRef, scrollIntoView, resetHeight, scrollViewRef } =
    useScrollPositionContext();

  React.useLayoutEffect(() => {
    if (!content || !userTriggeredExpand) {
      return;
    }

    const viewportBottom = scrollViewRef.current?.getScrollOffsetHeight() ?? window.innerHeight;
    const { scrollHeight } = content;

    const rect = content.getBoundingClientRect();

    const predictedRect = new DOMRect(rect.left, rect.top, rect.width, scrollHeight);

    // If the content is larger than the viewport we need to pre-allocate space for it and scroll to the header.
    // We don't scroll to the bottom of the content in this case to keep the header the user interacted with in view.
    const contentIsLargerThanViewport = scrollHeight > viewportBottom;
    if (contentIsLargerThanViewport) {
      spacePreAllocationViewRef.current?.allocate(viewportBottom + predictedRect.height);
      headerRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
      resetHeight();
      return;
    }

    // If the content smaller than the viewport but the bottom of the content is predicted to be below the viewport, scroll to the bottom of the content.
    const contentIsBelowViewport = predictedRect.bottom > viewportBottom;
    if (contentIsBelowViewport) {
      spacePreAllocationViewRef.current?.allocate(viewportBottom + predictedRect.height);
      scrollIntoView(predictedRect);
      resetHeight();
    }

    setUserTriggeredExpand(false);
  }, [
    content,
    headerRef,
    resetHeight,
    scrollIntoView,
    scrollViewRef,
    setUserTriggeredExpand,
    spacePreAllocationViewRef,
    userTriggeredExpand,
  ]);
}
