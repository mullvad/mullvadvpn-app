import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import { Ownership } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { FilterChip, Flex, Icon, IconButton, LabelTiny } from '../../lib/components';
import { useRelaySettingsUpdater } from '../../lib/constraint-updater';
import { daitaFilterActive, filterSpecialLocations } from '../../lib/filter-locations';
import { Spacings } from '../../lib/foundations';
import { useHistory } from '../../lib/history';
import { formatHtml } from '../../lib/html-formatter';
import { useNormalRelaySettings } from '../../lib/relay-settings-hooks';
import { RoutePath } from '../../lib/routes';
import { useSelector } from '../../redux/store';
import { AppNavigationHeader } from '../';
import * as Cell from '../cell';
import { useFilteredProviders } from '../Filter';
import { BackAction } from '../KeyboardNavigation';
import { Layout, SettingsContainer } from '../Layout';
import { NavigationContainer } from '../NavigationContainer';
import { NavigationScrollbars } from '../NavigationScrollbars';
import CombinedLocationList, { CombinedLocationListProps } from './CombinedLocationList';
import CustomLists from './CustomLists';
import { useRelayListContext } from './RelayListContext';
import { ScopeBarItem } from './ScopeBar';
import { useScrollPositionContext } from './ScrollPositionContext';
import {
  useOnSelectBridgeLocation,
  useOnSelectEntryLocation,
  useOnSelectExitLocation,
} from './select-location-hooks';
import { LocationType, SpecialBridgeLocationType, SpecialLocation } from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';
import {
  StyledContent,
  StyledDaitaSettingsButton,
  StyledNavigationBarAttachment,
  StyledScopeBar,
  StyledSearchBar,
  StyledSelectionUnavailable,
  StyledSelectionUnavailableText,
} from './SelectLocationStyles';
import { SpacePreAllocationView } from './SpacePreAllocationView';
import {
  AutomaticLocationRow,
  CustomBridgeLocationRow,
  CustomExitLocationRow,
} from './SpecialLocationList';

export default function SelectLocation() {
  const history = useHistory();
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const { saveScrollPosition, resetScrollPositions, scrollViewRef, spacePreAllocationViewRef } =
    useScrollPositionContext();
  const { locationType, setLocationType, setSearchTerm } = useSelectLocationContext();
  const { expandSearchResults } = useRelayListContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];
  const filteredProviders = useFilteredProviders(providers, ownership);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);
  const showDaitaFilter = daitaFilterActive(
    daita,
    directOnly,
    locationType,
    relaySettings?.tunnelProtocol ?? 'any',
    relaySettings?.wireguard.useMultihop ?? false,
  );

  const [searchValue, setSearchValue] = useState('');

  const onClose = useCallback(() => history.pop(), [history]);
  const onViewFilter = useCallback(() => history.push(RoutePath.filter), [history]);

  const tunnelProtocol = relaySettings?.tunnelProtocol ?? 'any';
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const allowEntrySelection =
    (tunnelProtocol === 'openvpn' && bridgeState === 'on') ||
    (tunnelProtocol !== 'openvpn' && relaySettings?.wireguard.useMultihop);

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
  const showFilters = showOwnershipFilter || showProvidersFilter || showDaitaFilter;
  return (
    <BackAction action={onClose}>
      <Layout>
        <SettingsContainer>
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
                    <ScopeBarItem>
                      {messages.pgettext('select-location-view', 'Entry')}
                    </ScopeBarItem>
                    <ScopeBarItem>{messages.pgettext('select-location-view', 'Exit')}</ScopeBarItem>
                  </StyledScopeBar>
                </>
              )}

              {locationType === LocationType.entry && daita && !directOnly ? null : (
                <>
                  {showFilters && (
                    <Flex
                      $gap={Spacings.spacing3}
                      $alignItems="center"
                      $flexWrap="wrap"
                      $margin={{ horizontal: Spacings.spacing3, bottom: Spacings.spacing5 }}>
                      <LabelTiny>
                        {messages.pgettext('select-location-view', 'Filtered:')}
                      </LabelTiny>

                      {showOwnershipFilter && (
                        <FilterChip
                          trailing={<Icon icon="cross" size="small" />}
                          aria-label={messages.gettext('Clear')}
                          onClick={onClearOwnership}>
                          {ownershipFilterLabel(ownership)}
                        </FilterChip>
                      )}

                      {showProvidersFilter && (
                        <FilterChip
                          trailing={<Icon icon="cross" size="small" />}
                          aria-label={messages.gettext('Clear')}
                          onClick={onClearProviders}>
                          {sprintf(
                            messages.pgettext(
                              'select-location-view',
                              'Providers: %(numberOfProviders)d',
                            ),
                            { numberOfProviders: filteredProviders.length },
                          )}
                        </FilterChip>
                      )}

                      {showDaitaFilter && (
                        <FilterChip as="div">
                          {sprintf(
                            messages.pgettext('select-location-view', 'Setting: %(settingName)s'),
                            { settingName: 'DAITA' },
                          )}
                        </FilterChip>
                      )}
                    </Flex>
                  )}

                  <StyledSearchBar searchTerm={searchValue} onSearch={updateSearchTerm} />
                </>
              )}
            </StyledNavigationBarAttachment>

            <NavigationScrollbars ref={scrollViewRef}>
              <SpacePreAllocationView ref={spacePreAllocationViewRef}>
                <StyledContent>
                  <SelectLocationContent />
                </StyledContent>
              </SpacePreAllocationView>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
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
  const { locationType, searchTerm } = useSelectLocationContext();
  const { selectedLocationRef, resetHeight } = useScrollPositionContext();
  const { relayList, expandLocation, collapseLocation, onBeforeExpand } = useRelayListContext();
  const [onSelectExitRelay, onSelectExitSpecial] = useOnSelectExitLocation();
  const [onSelectEntryRelay, onSelectEntrySpecial] = useOnSelectEntryLocation();
  const [onSelectBridgeRelay, onSelectBridgeSpecial] = useOnSelectBridgeLocation();

  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);

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
      <>
        <CustomLists selectedElementRef={selectedLocationRef} onSelect={onSelectExitRelay} />
        <LocationList
          key={locationType}
          relayLocations={relayList}
          specialLocations={specialLocations}
          selectedElementRef={selectedLocationRef}
          onSelectRelay={onSelectExitRelay}
          onSelectSpecial={onSelectExitSpecial}
          onExpand={expandLocation}
          onCollapse={collapseLocation}
          onWillExpand={onBeforeExpand}
          onTransitionEnd={resetHeight}
          allowAddToCustomList={allowAddToCustomList}
        />
        <NoSearchResult specialLocationsLength={specialLocations.length} />
      </>
    );
  } else if (relaySettings?.tunnelProtocol !== 'openvpn') {
    if (daita && !directOnly && relaySettings?.wireguard.useMultihop) {
      return <DisabledEntrySelection />;
    }

    return (
      <>
        <CustomLists selectedElementRef={selectedLocationRef} onSelect={onSelectEntryRelay} />
        <LocationList
          key={locationType}
          relayLocations={relayList}
          selectedElementRef={selectedLocationRef}
          onSelectRelay={onSelectEntryRelay}
          onSelectSpecial={onSelectEntrySpecial}
          onExpand={expandLocation}
          onCollapse={collapseLocation}
          onWillExpand={onBeforeExpand}
          onTransitionEnd={resetHeight}
          allowAddToCustomList={allowAddToCustomList}
        />
        <NoSearchResult specialLocationsLength={0} />
      </>
    );
  } else {
    // Add the "Automatic" item
    const specialList: Array<SpecialLocation<SpecialBridgeLocationType>> = [
      {
        label: messages.pgettext('select-location-view', 'Custom bridge'),
        value: SpecialBridgeLocationType.custom,
        selected: bridgeSettings?.type === 'custom',
        disabled: bridgeSettings?.custom === undefined,
        component: CustomBridgeLocationRow,
      },
      {
        label: messages.gettext('Automatic'),
        value: SpecialBridgeLocationType.closestToExit,
        selected: bridgeSettings?.type === 'normal' && bridgeSettings.normal?.location === 'any',
        component: AutomaticLocationRow,
      },
    ];

    const specialLocations = filterSpecialLocations(searchTerm, specialList);
    return (
      <>
        <CustomLists selectedElementRef={selectedLocationRef} onSelect={onSelectBridgeRelay} />
        <LocationList
          key={locationType}
          relayLocations={relayList}
          specialLocations={specialLocations}
          selectedElementRef={selectedLocationRef}
          onSelectRelay={onSelectBridgeRelay}
          onSelectSpecial={onSelectBridgeSpecial}
          onExpand={expandLocation}
          onCollapse={collapseLocation}
          onWillExpand={onBeforeExpand}
          onTransitionEnd={resetHeight}
          allowAddToCustomList={allowAddToCustomList}
        />
        <NoSearchResult specialLocationsLength={specialLocations.length} />
      </>
    );
  }
}

function LocationList<T>(props: CombinedLocationListProps<T>) {
  const { searchTerm } = useSelectLocationContext();

  if (
    searchTerm !== '' &&
    !props.relayLocations.some((country) => country.visible) &&
    (props.specialLocations === undefined || props.specialLocations.length === 0)
  ) {
    return null;
  } else {
    return (
      <>
        <Cell.Row>
          <Cell.Label>{messages.pgettext('select-location-view', 'All locations')}</Cell.Label>
        </Cell.Row>
        <CombinedLocationList {...props} />
      </>
    );
  }
}

interface NoSearchResultProps {
  specialLocationsLength: number;
}

function NoSearchResult(props: NoSearchResultProps) {
  const { relayList, customLists } = useRelayListContext();
  const { searchTerm } = useSelectLocationContext();

  if (
    searchTerm === '' ||
    relayList.some((country) => country.visible) ||
    customLists.some((list) => list.visible) ||
    props.specialLocationsLength > 0
  ) {
    return null;
  }

  return (
    <StyledSelectionUnavailable>
      <StyledSelectionUnavailableText>
        {formatHtml(
          sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), {
            searchTerm,
          }),
        )}
      </StyledSelectionUnavailableText>
      <StyledSelectionUnavailableText>
        {messages.gettext('Try a different search.')}
      </StyledSelectionUnavailableText>
    </StyledSelectionUnavailable>
  );
}

function DisabledEntrySelection() {
  const { push } = useHistory();

  const multihop = messages.pgettext('settings-view', 'Multihop');
  const directOnly = messages.gettext('Direct only');

  const navigateToDaitaSettings = useCallback(() => {
    push(RoutePath.daitaSettings);
  }, [push]);

  return (
    <StyledSelectionUnavailable>
      <StyledSelectionUnavailableText>
        {sprintf(
          messages.pgettext(
            'select-location-view',
            'The entry server for %(multihop)s is currently overridden by %(daita)s. To select an entry server, please first enable “%(directOnly)s” or disable "%(daita)s" in the settings.',
          ),
          { daita: strings.daita, multihop, directOnly },
        )}
      </StyledSelectionUnavailableText>
      <StyledDaitaSettingsButton onClick={navigateToDaitaSettings}>
        {sprintf(messages.pgettext('select-location-view', 'Open %(daita)s settings'), {
          daita: strings.daita,
        })}
      </StyledDaitaSettingsButton>
    </StyledSelectionUnavailable>
  );
}
