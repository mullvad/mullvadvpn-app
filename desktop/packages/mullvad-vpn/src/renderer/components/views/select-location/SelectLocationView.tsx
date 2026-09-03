import React from 'react';
import styled, { css } from 'styled-components';

import { LocationType } from '../../../features/locations/types';
import { usePrevious } from '../../../hooks';
import { Carousel } from '../../../lib/components/carousel';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import type { IScrollEvent } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  LocationLists,
  SelectLocationHeader,
  SelectLocationSelector,
  SpacePreAllocationView,
} from './components';
import { useMeasureExpandedLocationSelector, useMeasureIsolatedLocationSelector } from './hooks';
import { ScrollPositionContextProvider, useScrollPositionContext } from './ScrollPositionContext';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';

const StyledHeaderMaxHeightContainer = styled.div<{ $height: number; $previousHeight: number }>`
  ${({ $height, $previousHeight }) => css`
    --transition-duration: 0.25s;

    pointer-events: none;
    position: sticky;
    top: 0;
    z-index: 20;
    width: 100%;
    height: ${$height}px;
    background-color: transparent;

    transition: height var(--transition-duration) ease-in-out;

    ${() => {
      if ($previousHeight === 0) {
        return css`
          --transition-duration: 0;
        `;
      }
      return null;
    }}
  `}
`;

const StyledHeaderContainer = styled.div`
  pointer-events: auto;
  background-color: ${colors.darkBlue};
`;

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { setScrollTop, scrollViewRef, spacePreAllocationViewRef } = useScrollPositionContext();
  const { locationType, isolatedItem } = useSelectLocationViewContext();
  const [slideIndex, setSlideIndex] = React.useState(locationType === LocationType.entry ? 0 : 1);
  const [changingSlide, setChangingSlide] = React.useState(false);

  React.useLayoutEffect(() => {
    setChangingSlide(true);
    setSlideIndex(locationType === LocationType.entry ? 0 : 1);
  }, [locationType]);

  const handleSlideSettled = React.useCallback(() => {
    setChangingSlide(false);
  }, []);

  const onClose = React.useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      setScrollTop(event.scrollTop);
    },
    [setScrollTop],
  );

  const {
    singlehopElement,
    multihopElement,
    height: expandedElementHeight,
  } = useMeasureExpandedLocationSelector();
  const { element: isolatedElement, height: isolatedElementHeight } =
    useMeasureIsolatedLocationSelector();

  const height = isolatedItem ? isolatedElementHeight : expandedElementHeight;
  const previousHeight = usePrevious(height);

  return (
    <View backgroundColor="darkBlue">
      {singlehopElement}
      {multihopElement}
      {isolatedElement}
      <BackAction action={onClose}>
        <NavigationContainer>
          <NavigationScrollbars
            ref={scrollViewRef}
            onScroll={handleScroll}
            scrollPadding={`${height}px 0 0 0`}>
            <StyledHeaderMaxHeightContainer $height={height} $previousHeight={previousHeight}>
              <StyledHeaderContainer>
                <SelectLocationHeader>
                  <SelectLocationSelector />
                </SelectLocationHeader>
              </StyledHeaderContainer>
            </StyledHeaderMaxHeightContainer>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <Carousel
                    disableScroll
                    slideIndex={slideIndex}
                    onSlideIndexChange={setSlideIndex}
                    onSlideSettled={handleSlideSettled}>
                    <Carousel.Slides>
                      <Carousel.Slides.Slide key="entry">
                        {(changingSlide || locationType === LocationType.entry) && (
                          <LocationLists type={LocationType.entry} />
                        )}
                      </Carousel.Slides.Slide>
                      <Carousel.Slides.Slide key="exit">
                        {(changingSlide || locationType === LocationType.exit) && (
                          <LocationLists type={LocationType.exit} />
                        )}
                      </Carousel.Slides.Slide>
                    </Carousel.Slides>
                  </Carousel>
                </View.Container>
              </SpacePreAllocationView>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <ScrollPositionContextProvider>
        <SelectLocationViewImpl />
      </ScrollPositionContextProvider>
    </SelectLocationViewProvider>
  );
}
