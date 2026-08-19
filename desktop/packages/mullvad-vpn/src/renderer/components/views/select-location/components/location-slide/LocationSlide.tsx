import React from 'react';
import styled, { css } from 'styled-components';

import { LocationType } from '../../../../../features/locations/types';
import { usePrevious } from '../../../../../hooks';
import { Carousel } from '../../../../../lib/components/carousel';
import { useCarouselContext } from '../../../../../lib/components/carousel/CarouselContext';
import { View } from '../../../../../lib/components/view';
import { spacings } from '../../../../../lib/foundations';
import type { IScrollEvent } from '../../../../CustomScrollbars';
import { NavigationScrollbars } from '../../../../NavigationScrollbars';
import { useIsLocationSelectorIsolated } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { LocationLists } from '../location-lists';
import { LocationSlideContextProvider, useLocationSlideContext } from './LocationSlideContext';

export type LocationSlideProps = {
  collapsedHeaderHeight: number;
  expandedHeaderHeight: number;
  index: number;
  type: LocationType;
};

const StyledCarouselSlide = styled(Carousel.Slides.Slide)<{
  $containerOffset: number;
}>`
  ${({ $containerOffset }) => {
    return css`
      // TODO: These 48px are added to compensate for the height added by AppNavigationHeader
      // they should not be added statically like this
      height: calc(100vh - ${$containerOffset}px - 48px);
      margin-top: ${$containerOffset}px;
      // NOTE: Keep padding-top in sync with StyledLocationSelectorContainer's padding-bottom
      padding-top: ${spacings.small};
    `;
  }}
`;

const StyledStickyContainer = styled.div<{
  $marginTop: number;
}>`
  ${({ $marginTop }) => {
    return css`
      margin-top: ${$marginTop}px;
    `;
  }}
`;

function LocationSlideImpl({
  collapsedHeaderHeight,
  expandedHeaderHeight,
  type,
  index,
}: LocationSlideProps) {
  const { resetScroll, scrollViewRef } = useLocationSlideContext();
  const { setScrollTop } = useSelectLocationViewContext();
  const { slideIndex } = useCarouselContext();

  const [canResetScroll, setCanResetScroll] = React.useState(slideIndex === index);
  const previousSlideIndex = usePrevious(slideIndex);

  const slideIndexChanged = previousSlideIndex !== slideIndex;
  const headerHeightDiff = expandedHeaderHeight - collapsedHeaderHeight;

  React.useEffect(() => {
    if (slideIndexChanged && slideIndex === index) {
      setCanResetScroll(true);
    }
  }, [expandedHeaderHeight, index, slideIndex, slideIndexChanged]);

  React.useEffect(() => {
    if (canResetScroll) {
      resetScroll();
      setCanResetScroll(false);
    }
  }, [resetScroll, canResetScroll]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      if (slideIndex === index) {
        setScrollTop(event.scrollTop);
      }
    },
    [setScrollTop, slideIndex, index],
  );

  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  const containerOffset = isLocationSelectorIsolated ? 0 : collapsedHeaderHeight;
  const contentOffset = isLocationSelectorIsolated ? collapsedHeaderHeight : headerHeightDiff;

  return (
    <StyledCarouselSlide $containerOffset={containerOffset}>
      <NavigationScrollbars onScroll={handleScroll} ref={scrollViewRef} trackOffset={contentOffset}>
        <StyledStickyContainer $marginTop={contentOffset}>
          <View.Container horizontalMargin="medium" flexDirection="column">
            <LocationLists type={type} />
          </View.Container>
        </StyledStickyContainer>
      </NavigationScrollbars>
    </StyledCarouselSlide>
  );
}

export function LocationSlide({
  collapsedHeaderHeight,
  expandedHeaderHeight,
  index,
  type,
}: LocationSlideProps) {
  return (
    <LocationSlideContextProvider>
      <LocationSlideImpl
        collapsedHeaderHeight={collapsedHeaderHeight}
        expandedHeaderHeight={expandedHeaderHeight}
        index={index}
        type={type}
      />
    </LocationSlideContextProvider>
  );
}
