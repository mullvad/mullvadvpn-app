import { AnimatePresence } from 'motion/react';
import React, { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { LocationType } from '../../../features/locations/types';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useOnStableLayout } from '../../../lib/hooks';
import { AppNavigationHeader } from '../../';
import type { IScrollEvent } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  HeaderMenuIconButton,
  LocationLists,
  LocationListSlide,
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
  const { setScrollTop, scrollViewRef, spacePreAllocationViewRef, resetScroll } =
    useScrollPositionContext();
  const { locationType } = useSelectLocationViewContext();

  useOnStableLayout(resetScroll);

  const onClose = useCallback(() => history.pop(), [history]);

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      setScrollTop(event.scrollTop);
    },
    [setScrollTop],
  );

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
              </FlexColumn>
            </StyledStickyContainer>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <AnimatePresence mode="wait" initial={false}>
                    {locationType === LocationType.entry ? (
                      <LocationListSlide key="entry">
                        <LocationLists type={LocationType.entry} />
                      </LocationListSlide>
                    ) : (
                      <LocationListSlide key="exit">
                        <LocationLists type={LocationType.exit} />
                      </LocationListSlide>
                    )}
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
