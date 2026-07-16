import { useCallback } from 'react';

import { messages } from '../../../../shared/gettext';
import { useActiveFilters } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { useMultihop } from '../../../features/multihop/hooks';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import {
  FilterChips,
  HeaderMenuIconButton,
  LocationLists,
  LocationSearchField,
  ScopeBarItem,
  SpacePreAllocationView,
} from './components';
import { ScrollPositionContextProvider, useScrollPositionContext } from './ScrollPositionContext';
import { StyledScopeBar } from './SelectLocationStyles';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { saveScrollPosition, scrollViewRef, spacePreAllocationViewRef } =
    useScrollPositionContext();
  const { locationType, setLocationType } = useSelectLocationViewContext();

  const { multihop } = useMultihop();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  const onClose = useCallback(() => history.pop(), [history]);

  const changeLocationType = useCallback(
    (locationType: LocationType) => {
      saveScrollPosition();
      setLocationType(locationType);
    },
    [saveScrollPosition, setLocationType],
  );

  const showEntryExitBar = multihop === 'always';

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
            {showEntryExitBar && (
              <StyledScopeBar selectedIndex={locationType} onChange={changeLocationType}>
                <ScopeBarItem>{messages.pgettext('select-location-view', 'Entry')}</ScopeBarItem>
                <ScopeBarItem>{messages.pgettext('select-location-view', 'Exit')}</ScopeBarItem>
              </StyledScopeBar>
            )}
            {isAnyFilterActive && <FilterChips />}
            <LocationSearchField />
          </View.Container>

          <NavigationScrollbars ref={scrollViewRef}>
            <View.Content padding={{ top: 'small' }}>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <View.Container horizontalMargin="medium" flexDirection="column">
                  <LocationLists
                    // Set key to reset list when switching between entry and exit
                    key={locationType}
                    type={locationType}
                  />
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
