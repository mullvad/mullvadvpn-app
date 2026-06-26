import { AnimatePresence, motion } from 'motion/react';
import React, { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useActiveFilters } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import type { IScrollEvent } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  FilterChips,
  HeaderMenuIconButton,
  LocationLists,
  SelectLocationSelector,
  SpacePreAllocationView,
} from './components';
import { ScrollPositionContextProvider, useScrollPositionContext } from './ScrollPositionContext';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';

const StyledStickyContainer = styled.div`
  position: sticky;
  top: 0;
  z-index: 20;
  width: 100%;
  background-color: ${colors.darkBlue};
`;

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { setScrollTop, scrollViewRef, spacePreAllocationViewRef } = useScrollPositionContext();
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  const onClose = useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      setScrollTop(event.scrollTop);
    },
    [setScrollTop],
  );

  const slideIndex = locationType === LocationType.entry ? 0 : 1;

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={onClose}>
        <NavigationContainer>
          <NavigationScrollbars onScroll={handleScroll} ref={scrollViewRef}>
            <StyledStickyContainer>
              <AppNavigationHeader
                title={
                  // TRANSLATORS: Title label in navigation bar
                  messages.pgettext('select-location-nav', 'Select location')
                }
                titleVisible>
                <HeaderMenuIconButton />
              </AppNavigationHeader>
              <FlexColumn
                margin={{ horizontal: 'medium' }}
                padding={{ bottom: 'small' }}
                gap="small">
                <SelectLocationSelector />
                {isAnyFilterActive && <FilterChips />}
              </FlexColumn>
            </StyledStickyContainer>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <Carousel disableScroll slideIndex={slideIndex}>
                    <Carousel.Slides>
                      <Carousel.Slides.Slide key="entry">
                        <AnimatePresence>
                          {locationType === LocationType.entry && (
                            <motion.div
                              key="entry"
                              initial={{ opacity: 1 }}
                              exit={{ opacity: 0.4 }}
                              transition={{ duration: 0.2 }}>
                              <LocationLists type={LocationType.entry} />
                            </motion.div>
                          )}
                        </AnimatePresence>
                      </Carousel.Slides.Slide>
                      <Carousel.Slides.Slide key="exit">
                        <AnimatePresence>
                          {locationType === LocationType.exit && (
                            <motion.div
                              key="exit"
                              initial={{ opacity: 1 }}
                              exit={{ opacity: 0.4 }}
                              transition={{ duration: 0.2 }}>
                              <LocationLists type={LocationType.exit} />
                            </motion.div>
                          )}
                        </AnimatePresence>
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
