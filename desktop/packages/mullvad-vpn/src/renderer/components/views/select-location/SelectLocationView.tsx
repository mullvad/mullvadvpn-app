import { AnimatePresence, type AnimationDefinition } from 'motion/react';
import React from 'react';
import styled, { css } from 'styled-components';

import { usePrevious } from '../../../hooks';
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
import { LocationSlide } from './components/location-slide/LocationSlide';
import { useMeasureExpandedLocationSelector, useMeasureIsolatedLocationSelector } from './hooks';
import { ScrollPositionContextProvider, useScrollPositionContext } from './ScrollPositionContext';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';
import { shouldLocationSelectorExpand } from './utils';

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
  const { scrollViewRef, spacePreAllocationViewRef } = useScrollPositionContext();
  const { locationType, isolatedItem, setIsLocationSelectorExpanded } =
    useSelectLocationViewContext();

  const [showScrollbar, setShowScrolllbar] = React.useState(true);

  const onClose = React.useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      const shouldExpand = shouldLocationSelectorExpand(event.scrollTop);
      setIsLocationSelectorExpanded(shouldExpand);
    },
    [setIsLocationSelectorExpanded],
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

  const handleAnimationStart = React.useCallback(
    (definition: AnimationDefinition) => {
      if (typeof definition === 'object' && 'opacity' in definition) {
        if (definition.opacity === 0) {
          setShowScrolllbar(false);
        }
      }
    },
    [setShowScrolllbar],
  );

  const handleExitComplete = React.useCallback(() => {
    setShowScrolllbar(true);
  }, []);

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
            trackPadding={{ x: 0, y: height }}
            showScrollIndicators={showScrollbar}>
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
                  <AnimatePresence mode="wait" onExitComplete={handleExitComplete}>
                    <LocationSlide
                      key={`${locationType}-location-lists`}
                      onAnimationStart={handleAnimationStart}>
                      <LocationLists type={locationType} />
                    </LocationSlide>
                  </AnimatePresence>
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
