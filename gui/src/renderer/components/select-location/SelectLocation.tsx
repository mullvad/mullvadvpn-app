import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { colors } from '../../../config.json';
import { Ownership } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useRelaySettingsUpdater } from '../../lib/constraint-updater';
import { daitaFilterActive, filterSpecialLocations } from '../../lib/filter-locations';
import { useHistory } from '../../lib/history';
import { formatHtml } from '../../lib/html-formatter';
import { RoutePath } from '../../lib/routes';
import { useNormalRelaySettings, useStyledRef } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import * as Cell from '../cell';
import { useFilteredProviders } from '../Filter';
import ImageView from '../ImageView';
import { BackAction } from '../KeyboardNavigation';
import { Layout, SettingsContainer } from '../Layout';
import {
  NavigationBar,
  NavigationBarButton,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from '../NavigationBar';
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
  StyledClearFilterButton,
  StyledContent,
  StyledFilter,
  StyledFilterRow,
  StyledHeaderSubTitle,
  StyledNavigationBarAttachment,
  StyledNoResult,
  StyledNoResultText,
  StyledScopeBar,
  StyledSearchBar,
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

  const searchInputRef = useStyledRef<HTMLInputElement>();

  const changeLocationType = useCallback(
    (locationType: LocationType) => {
      searchInputRef.current?.focus();
      saveScrollPosition();
      setLocationType(locationType);
    },
    [saveScrollPosition],
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
    [resetScrollPositions, expandSearchResults],
  );

  const showOwnershipFilter = ownership !== Ownership.any;
  const showProvidersFilter = providers.length > 0;
  const showFilters = showOwnershipFilter || showProvidersFilter || showDaitaFilter;

  return (
    <BackAction action={onClose}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar alwaysDisplayBarTitle>
              <NavigationItems>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('select-location-nav', 'Select location')
                  }
                </TitleBarItem>

                <NavigationBarButton onClick={onViewFilter} aria-label={messages.gettext('Filter')}>
                  <ImageView
                    source="icon-filter-round"
                    tintColor={colors.white40}
                    tintHoverColor={colors.white60}
                    height={24}
                    width={24}
                  />
                </NavigationBarButton>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationBarAttachment>
              {allowEntrySelection && (
                <>
                  <StyledScopeBar selectedIndex={locationType} onChange={changeLocationType}>
                    <ScopeBarItem>
                      {messages.pgettext('select-location-view', 'Entry')}
                    </ScopeBarItem>
                    <ScopeBarItem>{messages.pgettext('select-location-view', 'Exit')}</ScopeBarItem>
                  </StyledScopeBar>

                  {tunnelProtocol === 'openvpn' ? (
                    <StyledHeaderSubTitle>
                      {messages.pgettext(
                        'select-location-view',
                        'While connected, your traffic will be routed through two secure locations, the entry point (a bridge server) and the exit point (a VPN server).',
                      )}
                    </StyledHeaderSubTitle>
                  ) : (
                    <StyledHeaderSubTitle>
                      {messages.pgettext(
                        'select-location-view',
                        'While connected, your traffic will be routed through two secure locations, the entry point and the exit point (needs to be two different VPN servers).',
                      )}
                    </StyledHeaderSubTitle>
                  )}
                </>
              )}

              {showFilters && (
                <StyledFilterRow>
                  {messages.pgettext('select-location-view', 'Filtered:')}

                  {showOwnershipFilter && (
                    <StyledFilter>
                      {ownershipFilterLabel(ownership)}
                      <StyledClearFilterButton
                        aria-label={messages.gettext('Clear')}
                        onClick={onClearOwnership}>
                        <ImageView
                          height={16}
                          width={16}
                          source="icon-close"
                          tintColor={colors.white60}
                          tintHoverColor={colors.white80}
                        />
                      </StyledClearFilterButton>
                    </StyledFilter>
                  )}

                  {showProvidersFilter && (
                    <StyledFilter>
                      {sprintf(
                        messages.pgettext(
                          'select-location-view',
                          'Providers: %(numberOfProviders)d',
                        ),
                        { numberOfProviders: filteredProviders.length },
                      )}
                      <StyledClearFilterButton
                        aria-label={messages.gettext('Clear')}
                        onClick={onClearProviders}>
                        <ImageView
                          height={16}
                          width={16}
                          source="icon-close"
                          tintColor={colors.white60}
                          tintHoverColor={colors.white80}
                        />
                      </StyledClearFilterButton>
                    </StyledFilter>
                  )}

                  {showDaitaFilter && (
                    <StyledFilter>
                      {sprintf(
                        messages.pgettext('select-location-view', 'Setting: %(settingName)s'),
                        { settingName: 'DAITA' },
                      )}
                    </StyledFilter>
                  )}
                </StyledFilterRow>
              )}

              <StyledSearchBar
                searchInputRef={searchInputRef}
                searchTerm={searchValue}
                onSearch={updateSearchTerm}
              />
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
    <StyledNoResult>
      <StyledNoResultText>
        {formatHtml(
          sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), {
            searchTerm,
          }),
        )}
      </StyledNoResultText>
      <StyledNoResultText>{messages.gettext('Try a different search.')}</StyledNoResultText>
    </StyledNoResult>
  );
}
