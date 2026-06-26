import { AnimatePresence, motion } from 'motion/react';
import { useCallback } from 'react';
import React from 'react';

import { messages } from '../../../../shared/gettext';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../../features/daita/hooks';
import { useActiveFilters } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { useMultihop } from '../../../features/multihop/hooks';
import { Carousel } from '../../../lib/components/carousel';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import type { IScrollEvent } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  DisabledEntrySelection,
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

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { scrollViewRef, spacePreAllocationViewRef, setScrollTop } = useScrollPositionContext();
  const { locationType } = useSelectLocationViewContext();
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { multihop } = useMultihop();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  const onClose = useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      setScrollTop(event.scrollTop);
    },
    [setScrollTop],
  );

  const showDisabledEntrySelection =
    locationType === LocationType.entry && daitaEnabled && !daitaDirectOnly && multihop;
  const showFilters = isAnyFilterActive && !showDisabledEntrySelection;
  const slideIndex = locationType === LocationType.entry ? 0 : 1;

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={onClose}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('select-location-nav', 'Select location')
            }
            titleVisible>
            <HeaderMenuIconButton />
          </AppNavigationHeader>

          <View.Container
            flexDirection="column"
            horizontalMargin="medium"
            padding={{ bottom: 'small' }}>
            <SelectLocationSelector />

            {showFilters && <FilterChips />}
          </View.Container>

          <NavigationScrollbars onScroll={handleScroll} ref={scrollViewRef}>
            <View.Content padding={{ top: 'small' }}>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <Carousel slideIndex={slideIndex}>
                    <Carousel.Slides>
                      <Carousel.Slides.Slide key="entry">
                        <AnimatePresence>
                          {locationType === LocationType.entry && (
                            <motion.div
                              key="entry"
                              initial={{ opacity: 1 }}
                              exit={{ opacity: 0.4 }}
                              transition={{ duration: 0.2 }}>
                              {showDisabledEntrySelection ? (
                                <DisabledEntrySelection />
                              ) : (
                                <LocationLists type={LocationType.entry} />
                              )}
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
