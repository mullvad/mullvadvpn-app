import { AnimatePresence, motion } from 'motion/react';
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

const StyledHeaderMaxHeightContainer = styled(motion.div)`
  pointer-events: none;
  position: sticky;
  top: 0;
  z-index: 20;
  width: 100%;
  height: var(--header-height);
  background-color: transparent;
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
            showScrollIndicators={
              transitionState === 'transitioningOut' || transitionState === 'transitioningIn'
                ? false
                : undefined
            }>
            <StyledHeaderMaxHeightContainer
              initial={false}
              animate={{ height }}
              transition={{
                height: { duration: previousHeight === 0 ? 0 : 0.15 },
              }}>
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
