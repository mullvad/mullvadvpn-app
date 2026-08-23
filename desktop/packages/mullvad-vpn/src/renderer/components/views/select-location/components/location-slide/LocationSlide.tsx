import React from 'react';
import styled, { css } from 'styled-components';

import { LocationType } from '../../../../../features/locations/types';
import { Carousel } from '../../../../../lib/components/carousel';
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
  navigationHeaderHeight: number;
  slideActive: boolean;
  type: LocationType;
};

const StyledCarouselSlide = styled(Carousel.Slides.Slide)<{
  $containerOffset: number;
  $navigationHeaderHeight: number;
}>`
  ${({ $containerOffset, $navigationHeaderHeight }) => {
    return css`
      height: calc(100vh - ${$containerOffset}px - ${$navigationHeaderHeight}px);
      margin-top: ${$containerOffset}px;
      // NOTE: Keep padding-top in sync with padding-bottom in the StyledLocationSelectorInnerContainer
      // styled component in the SelectLocationView component.
      padding-top: ${spacings.small};
    `;
  }}
`;

const StyledContentContainer = styled.div<{
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
  navigationHeaderHeight,
  slideActive,
  type,
}: LocationSlideProps) {
  const { resetScroll, scrollViewRef } = useLocationSlideContext();
  const { setScrollTop } = useSelectLocationViewContext();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  const headerHeightDiff = expandedHeaderHeight - collapsedHeaderHeight;
  const containerOffset = isLocationSelectorIsolated ? 0 : collapsedHeaderHeight;
  const contentOffset = isLocationSelectorIsolated ? collapsedHeaderHeight : headerHeightDiff;

  const headersCalculated =
    collapsedHeaderHeight > 0 && expandedHeaderHeight > 0 && navigationHeaderHeight > 0;
  React.useEffect(() => {
    if (slideActive && headersCalculated) {
      resetScroll();
    }
  }, [resetScroll, slideActive, headersCalculated]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      if (slideActive) {
        setScrollTop(event.scrollTop);
      }
    },
    [setScrollTop, slideActive],
  );

  return (
    <StyledCarouselSlide
      $containerOffset={containerOffset}
      $navigationHeaderHeight={navigationHeaderHeight}>
      <NavigationScrollbars onScroll={handleScroll} ref={scrollViewRef} trackOffset={contentOffset}>
        <StyledContentContainer $marginTop={contentOffset}>
          <View.Container horizontalMargin="medium" flexDirection="column">
            <LocationLists type={type} />
          </View.Container>
        </StyledContentContainer>
      </NavigationScrollbars>
    </StyledCarouselSlide>
  );
}

export function LocationSlide({
  collapsedHeaderHeight,
  expandedHeaderHeight,
  navigationHeaderHeight,
  slideActive,
  type,
}: LocationSlideProps) {
  return (
    <LocationSlideContextProvider>
      <LocationSlideImpl
        collapsedHeaderHeight={collapsedHeaderHeight}
        expandedHeaderHeight={expandedHeaderHeight}
        navigationHeaderHeight={navigationHeaderHeight}
        slideActive={slideActive}
        type={type}
      />
    </LocationSlideContextProvider>
  );
}
