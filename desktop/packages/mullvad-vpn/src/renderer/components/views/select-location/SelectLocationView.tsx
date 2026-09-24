import { AnimatePresence } from 'motion/react';
import React from 'react';
import styled, { css } from 'styled-components';

import { usePrevious } from '../../../hooks';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { type IScrollEvent, StyledScrollable } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  SelectLocationHeader,
  SelectLocationSelector,
  SpacePreAllocationView,
  StyledSelectLocationHeader,
} from './components';
import { useLocationSlides, useMeasureLocationSelector } from './hooks';
import { ScrollPositionContextProvider, useScrollPositionContext } from './ScrollPositionContext';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';
import { shouldLocationSelectorExpand } from './utils';

const StyledView = styled(View)<{ $headerHeight: number }>`
  ${({ $headerHeight }) => css`
    --header-height: ${$headerHeight}px;
  `}
`;

const StyledHeaderMaxHeightContainer = styled.div<{ $previousHeight: number }>`
  ${({ $previousHeight }) => css`
    --transition-duration: 0.25s;

    pointer-events: none;
    position: sticky;
    top: 0;
    z-index: 20;
    width: 100%;
    height: var(--header-height);
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

const StyledNavigationScrollbars = styled(NavigationScrollbars)`
  & ${StyledScrollable} {
    height: 100vh;
    scroll-padding-top: var(--header-height);
  }
  &:has(${StyledSelectLocationHeader}:focus-within) {
    & ${StyledScrollable} {
      scroll-padding-top: 0;
    }
  }
`;

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { scrollViewRef, spacePreAllocationViewRef } = useScrollPositionContext();
  const { setIsLocationSelectorExpanded, transitionState } = useSelectLocationViewContext();

  const onClose = React.useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      const shouldExpand = shouldLocationSelectorExpand(event.scrollTop);
      setIsLocationSelectorExpanded(shouldExpand);
    },
    [setIsLocationSelectorExpanded],
  );

  const { measureElement, height } = useMeasureLocationSelector();

  const previousHeight = usePrevious(height);
  const locationSlide = useLocationSlides();

  return (
    <StyledView backgroundColor="darkBlue" $headerHeight={height}>
      {measureElement}
      <BackAction action={onClose}>
        <NavigationContainer>
          <StyledNavigationScrollbars
            ref={scrollViewRef}
            onScroll={handleScroll}
            trackPadding={{ x: 0, y: height }}
            showScrollIndicators={transitionState === 'transitioningOut' ? false : undefined}>
            <StyledHeaderMaxHeightContainer $previousHeight={previousHeight}>
              <StyledHeaderContainer>
                <SelectLocationHeader>
                  <SelectLocationSelector />
                </SelectLocationHeader>
              </StyledHeaderContainer>
            </StyledHeaderMaxHeightContainer>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container
                  horizontalMargin="medium"
                  flexDirection="column"
                  flexGrow={1}
                  padding={{ top: 'tiny' }}>
                  <AnimatePresence mode="wait">{locationSlide}</AnimatePresence>
                </View.Container>
              </SpacePreAllocationView>
            </View.Content>
          </StyledNavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </StyledView>
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
