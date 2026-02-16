import { useCallback, useState } from 'react';

import { ObfuscationType, Ownership } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { useObfuscation } from '../../../features/anti-censorship/hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../../features/daita/hooks';
import { Container, Flex, IconButton, LabelTinySemiBold } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import {
  daitaFilterActive,
  filterSpecialLocations,
  lwoFilterActive,
  quicFilterActive,
} from '../../../lib/filter-locations';
import { useHistory } from '../../../lib/history';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { SearchTextField } from '../../search-text-field';
import {
  CountryLocationList,
  CustomExitLocationRow,
  CustomListLocationList,
  DaitaFilterChip,
  DisabledEntrySelection,
  LwoFilterChip,
  NoSearchResult,
  OwnershipFilterChip,
  ProvidersFilterChip,
  QuicFilterChip,
  ScopeBarItem,
  SpacePreAllocationView,
} from './components';
import { useHandleSelectEntryLocation, useHandleSelectExitLocation } from './hooks';
import { useScrollPositionContext } from './ScrollPositionContext';
import { LocationType, SpecialLocation } from './select-location-types';
import {
  StyledContent,
  StyledNavigationBarAttachment,
  StyledScopeBar,
} from './SelectLocationStyles';
import { useSelectLocationViewContext } from './SelectLocationViewContext';

export function SelectLocation() {
  const history = useHistory();
  const { saveScrollPosition, resetScrollPositions, scrollViewRef, spacePreAllocationViewRef } =
    useScrollPositionContext();
  const { locationType, setLocationType, setSearchTerm } = useSelectLocationViewContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();
  const { obfuscation } = useObfuscation();

  const showQuicFilter = quicFilterActive(
    obfuscation === ObfuscationType.quic,
    locationType,
    multihop,
  );
  const showLwoFilter = lwoFilterActive(
    obfuscation === ObfuscationType.lwo,
    locationType,
    multihop,
  );

  const showDaitaFilter = daitaFilterActive(daitaEnabled, daitaDirectOnly, locationType, multihop);

  const [searchValue, setSearchValue] = useState<string | undefined>(undefined);

  const onClose = useCallback(() => history.pop(), [history]);
  const onViewFilter = useCallback(() => history.push(RoutePath.filter), [history]);

  const allowEntrySelection = relaySettings?.wireguard.useMultihop;

  const changeLocationType = useCallback(
    (locationType: LocationType) => {
      saveScrollPosition();
      setLocationType(locationType);
    },
    [saveScrollPosition, setLocationType],
  );

  const updateSearchTerm = useCallback(
    (value: string) => {
      setSearchValue(value);
      if (value.length === 1) {
        setSearchTerm('');
      } else {
        resetScrollPositions();
        setSearchTerm(value);
      }
    },
    [setSearchTerm, resetScrollPositions],
  );

  const showOwnershipFilter = ownership !== Ownership.any;
  const showProvidersFilter = providers.length > 0;
  const showFilters =
    showOwnershipFilter ||
    showProvidersFilter ||
    showDaitaFilter ||
    showQuicFilter ||
    showLwoFilter;

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
            <IconButton
              variant="secondary"
              onClick={onViewFilter}
              aria-label={messages.gettext('Filter')}>
              <IconButton.Icon icon="filter-circle" />
            </IconButton>
          </AppNavigationHeader>

          <StyledNavigationBarAttachment>
            {allowEntrySelection && (
              <>
                <StyledScopeBar selectedIndex={locationType} onChange={changeLocationType}>
                  <ScopeBarItem>{messages.pgettext('select-location-view', 'Entry')}</ScopeBarItem>
                  <ScopeBarItem>{messages.pgettext('select-location-view', 'Exit')}</ScopeBarItem>
                </StyledScopeBar>
              </>
            )}

            {locationType === LocationType.entry && daitaEnabled && !daitaDirectOnly ? null : (
              <>
                {showFilters && (
                  <Flex
                    gap="small"
                    alignItems="center"
                    flexWrap="wrap"
                    margin={{ horizontal: 'small', bottom: 'medium' }}>
                    <LabelTinySemiBold>
                      {messages.pgettext('select-location-view', 'Filtered:')}
                    </LabelTinySemiBold>

                    {showOwnershipFilter && <OwnershipFilterChip />}
                    {showProvidersFilter && <ProvidersFilterChip />}
                    {showDaitaFilter && <DaitaFilterChip />}
                    {showQuicFilter && <QuicFilterChip />}
                    {showLwoFilter && <LwoFilterChip />}
                  </Flex>
                )}

                <SearchTextField
                  variant="secondary"
                  value={searchValue}
                  onValueChange={updateSearchTerm}>
                  <SearchTextField.Icon icon="search" />
                  <SearchTextField.Input
                    autoFocus
                    placeholder={
                      // TRANSLATORS: Placeholder text for search field in select location view
                      messages.gettext('Search locations or servers')
                    }
                  />
                  <SearchTextField.ClearButton />
                </SearchTextField>
              </>
            )}
          </StyledNavigationBarAttachment>

          <NavigationScrollbars ref={scrollViewRef}>
            <View.Content>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <StyledContent>
                  <SelectLocationContent />
                </StyledContent>
              </SpacePreAllocationView>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

function SelectLocationContent() {
  const { locationType, searchTerm } = useSelectLocationViewContext();
  const { selectedLocationRef } = useScrollPositionContext();

  const handleSelectExitLocation = useHandleSelectExitLocation();
  const handleSelectEntryLocation = useHandleSelectEntryLocation();

  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly } = useDaitaDirectOnly();

  const relaySettings = useNormalRelaySettings();

  if (locationType === LocationType.exit) {
    // Add "Custom" item if a custom relay is selected
    const specialList: Array<SpecialLocation<undefined>> = [];
    if (relaySettings === undefined) {
      specialList.push({
        label: messages.gettext('Custom'),
        value: undefined,
        selected: true,
        component: CustomExitLocationRow,
      });
    }

    const specialLocations = filterSpecialLocations(searchTerm, specialList);
    return (
      <Container horizontalMargin="medium" flexDirection="column" gap="large">
        <CustomListLocationList locationSelection="exit" selectedElementRef={selectedLocationRef} />
        <CountryLocationList
          key={locationType}
          selectedElementRef={selectedLocationRef}
          onSelect={handleSelectExitLocation}
        />
        <NoSearchResult specialLocationsLength={specialLocations.length} />
      </Container>
    );
  } else {
    if (daitaEnabled && !daitaDirectOnly && relaySettings?.wireguard.useMultihop) {
      return <DisabledEntrySelection />;
    }

    return (
      <Container horizontalMargin="medium" flexDirection="column" gap="large">
        <CustomListLocationList
          locationSelection="entry"
          selectedElementRef={selectedLocationRef}
        />
        <CountryLocationList
          key={locationType}
          selectedElementRef={selectedLocationRef}
          onSelect={handleSelectEntryLocation}
        />
        <NoSearchResult specialLocationsLength={0} />
      </Container>
    );
  }
}
