import { useCallback, useState } from 'react';

import { ObfuscationType, Ownership } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
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
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { SearchTextField } from '../../search-text-field';
import {
  CustomExitLocationRow,
  CustomLists,
  DaitaFilterChip,
  DisabledEntrySelection,
  LocationList,
  LwoFilterChip,
  NoSearchResult,
  OwnershipFilterChip,
  ProvidersFilterChip,
  QuicFilterChip,
  ScopeBarItem,
  SpacePreAllocationView,
} from './components';
import { useOnSelectEntryLocation, useOnSelectExitLocation } from './hooks';
import { useRelayListContext } from './RelayListContext';
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
  const { expandSearchResults } = useRelayListContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);
  const quic = useSelector(
    (state) => state.settings.obfuscationSettings.selectedObfuscation === ObfuscationType.quic,
  );
  const lwo = useSelector(
    (state) => state.settings.obfuscationSettings.selectedObfuscation === ObfuscationType.lwo,
  );
  const showQuicFilter = quicFilterActive(quic, locationType, multihop);
  const showLwoFilter = lwoFilterActive(lwo, locationType, multihop);
  const showDaitaFilter = daitaFilterActive(daita, directOnly, locationType, multihop);

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
        expandSearchResults('');
        setSearchTerm('');
      } else {
        resetScrollPositions();
        expandSearchResults(value);
        setSearchTerm(value);
      }
    },
    [expandSearchResults, setSearchTerm, resetScrollPositions],
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

            {locationType === LocationType.entry && daita && !directOnly ? null : (
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
  const { relayList } = useRelayListContext();
  const [onSelectExitRelay] = useOnSelectExitLocation();
  const [onSelectEntryRelay] = useOnSelectEntryLocation();

  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const relaySettings = useNormalRelaySettings();

  const allowAddToCustomList = useSelector((state) => state.settings.customLists.length > 0);

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
        <CustomLists locationSelection="exit" selectedElementRef={selectedLocationRef} />
        <LocationList
          key={locationType}
          locations={relayList}
          selectedElementRef={selectedLocationRef}
          onSelect={onSelectExitRelay}
          allowAddToCustomList={allowAddToCustomList}
        />
        <NoSearchResult specialLocationsLength={specialLocations.length} />
      </Container>
    );
  } else {
    if (daita && !directOnly && relaySettings?.wireguard.useMultihop) {
      return <DisabledEntrySelection />;
    }

    return (
      <Container horizontalMargin="medium" flexDirection="column" gap="large">
        <CustomLists locationSelection="entry" selectedElementRef={selectedLocationRef} />
        <LocationList
          key={locationType}
          locations={relayList}
          selectedElementRef={selectedLocationRef}
          onSelect={onSelectEntryRelay}
          allowAddToCustomList={allowAddToCustomList}
        />
        <NoSearchResult specialLocationsLength={0} />
      </Container>
    );
  }
}
