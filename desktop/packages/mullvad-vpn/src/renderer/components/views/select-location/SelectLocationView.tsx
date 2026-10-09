import { AnimatePresence, motion } from 'motion/react';
import React from 'react';
import styled, { css } from 'styled-components';

import { View } from '../../../lib/components/view';
import { spacings } from '../../../lib/foundations';
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

const StyledHeaderMaxHeightContainer = styled(motion.div)`
  pointer-events: none;
  position: sticky;
  top: 0;
  z-index: 20;
  width: 100%;
  background-color: transparent;
  margin-bottom: ${spacings.small};
`;

const StyledNavigationScrollbars = styled(NavigationScrollbars)<{ $headerHeight: number }>`
  ${({ $headerHeight }) => css`
    & ${StyledScrollable} {
      height: 100vh;
      scroll-padding-top: ${$headerHeight}px;
    }
    &:has(${StyledSelectLocationHeader}:focus-within) {
      & ${StyledScrollable} {
        scroll-padding-top: 0;
      }
    }
  `}
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

  const locationSlide = useLocationSlides();

  return (
    <View backgroundColor="darkBlue">
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
            }
            $headerHeight={height}>
            {/* Height will be 0 on first render, and will then be measured.
              Skip rendering when height is 0 to prevent transition. */}
            {height !== 0 && (
              <StyledHeaderMaxHeightContainer
                animate={{ height }}
                transition={{ height: { duration: 0.25, ease: 'easeOut' } }}>
                <SelectLocationHeader>
                  <SelectLocationSelector />
                </SelectLocationHeader>
              </StyledHeaderMaxHeightContainer>
            )}
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
