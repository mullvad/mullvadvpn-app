import { AnimatePresence, motion } from 'motion/react';
import React, { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { LocationType } from '../../../features/locations/types';
import { useMeasure } from '../../../hooks';
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
  top: 0px;
  z-index: 20;
`;

const StyledLocationSelectorContainer = styled.div`
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  width: 100%;
  background-color: ${colors.darkBlue};
`;

const StyledLocationSelectorContainerWrapper = styled.div`
  position: relative;
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

  const [measureRefHidden, { height: heightHidden }] = useMeasure();

  return (
    <View backgroundColor="darkBlue">
      <FlexColumn
        ref={measureRefHidden}
        style={{ position: 'fixed', visibility: 'hidden', transform: 'translateX(-100%)' }}>
        <FlexColumn margin={{ horizontal: 'medium' }} padding={{ bottom: 'small' }}>
          <SelectLocationSelector expanded />
        </FlexColumn>
      </FlexColumn>
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
              <StyledLocationSelectorContainerWrapper>
                <StyledLocationSelectorContainer>
                  <FlexColumn margin={{ horizontal: 'medium' }} padding={{ bottom: 'small' }}>
                    <SelectLocationSelector />
                  </FlexColumn>
                </StyledLocationSelectorContainer>
              </StyledLocationSelectorContainerWrapper>
            </StyledStickyContainer>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <motion.div
                    animate={{ height: `${heightHidden}px` }}
                    transition={{ duration: 0.15 }}
                  />
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
