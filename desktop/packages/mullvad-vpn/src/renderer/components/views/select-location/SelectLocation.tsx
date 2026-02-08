import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { ObfuscationType, Ownership } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import {
  Container,
  FilterChip,
  Flex,
  IconButton,
  LabelTinySemiBold,
} from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
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
import { useFilteredProviders } from '../../views/filter/hooks';
import {
  CustomExitLocationRow,
  CustomLists,
  DisabledEntrySelection,
  LocationList,
  NoSearchResult,
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
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const { saveScrollPosition, resetScrollPositions, scrollViewRef, spacePreAllocationViewRef } =
    useScrollPositionContext();
  const { locationType, setLocationType, setSearchTerm } = useSelectLocationViewContext();
  const { expandSearchResults } = useRelayListContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const filteredProviders = useFilteredProviders(providers, ownership);
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

  const onClearProviders = useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await relaySettingsUpdater((settings) => ({ ...settings, providers: [] }));
    }
  }, [relaySettingsUpdater, resetScrollPositions, relaySettings]);

  const onClearOwnership = useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await relaySettingsUpdater((settings) => ({ ...settings, ownership: Ownership.any }));
    }
  }, [relaySettingsUpdater, resetScrollPositions, relaySettings]);

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

                    {showOwnershipFilter && (
                      <FilterChip aria-label={messages.gettext('Clear')} onClick={onClearOwnership}>
                        <FilterChip.Text>{ownershipFilterLabel(ownership)}</FilterChip.Text>
                        <FilterChip.Icon icon="cross" />
                      </FilterChip>
                    )}

                    {showProvidersFilter && (
                      <FilterChip aria-label={messages.gettext('Clear')} onClick={onClearProviders}>
                        <FilterChip.Text>
                          {sprintf(
                            messages.pgettext(
                              'select-location-view',
                              'Providers: %(numberOfProviders)d',
                            ),
                            { numberOfProviders: filteredProviders.length },
                          )}
                        </FilterChip.Text>
                        <FilterChip.Icon icon="cross" />
                      </FilterChip>
                    )}

                    {showDaitaFilter && (
                      <FilterChip as="div">
                        <FilterChip.Text>
                          {sprintf(
                            messages.pgettext('select-location-view', 'Setting: %(settingName)s'),
                            { settingName: 'DAITA' },
                          )}
                        </FilterChip.Text>
                      </FilterChip>
                    )}

                    {showQuicFilter && (
                      <FilterChip as="div">
                        <FilterChip.Text>
                          {sprintf(
                            // TRANSLATORS: Label for indicator that shows that obfuscation is being used as a filter.
                            // TRANSLATORS: Available placeholders:
                            // TRANSLATORS: %(obfuscation)s - type of obfuscation in use
                            messages.pgettext(
                              'select-location-view',
                              'Obfuscation: %(obfuscation)s',
                            ),
                            { obfuscation: strings.quic },
                          )}
                        </FilterChip.Text>
                      </FilterChip>
                    )}

                    {showLwoFilter && (
                      <FilterChip as="div">
                        <FilterChip.Text>
                          {sprintf(
                            // TRANSLATORS: Label for indicator that shows that obfuscation is being used as a filter.
                            // TRANSLATORS: Available placeholders:
                            // TRANSLATORS: %(obfuscation)s - type of obfuscation in use
                            messages.pgettext(
                              'select-location-view',
                              'Obfuscation: %(obfuscation)s',
                            ),
                            { obfuscation: strings.lwo },
                          )}
                        </FilterChip.Text>
                      </FilterChip>
                    )}
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

function ownershipFilterLabel(ownership: Ownership): string {
  switch (ownership) {
    case Ownership.mullvadOwned:
      return messages.pgettext('filter-view', 'Owned');
    case Ownership.rented:
      return messages.pgettext('filter-view', 'Rented');
    default:
      throw new Error('Only owned and rented should make label visible');
  }
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
