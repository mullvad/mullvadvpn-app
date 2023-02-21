import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { colors } from '../../../config.json';
import { Ownership } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useAppContext } from '../../context';
import { filterSpecialLocations } from '../../lib/filter-locations';
import { useHistory } from '../../lib/history';
import { formatHtml } from '../../lib/html-formatter';
import { RoutePath } from '../../lib/routes';
import { useNormalBridgeSettings, useNormalRelaySettings } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import ImageView from '../ImageView';
import { BackAction } from '../KeyboardNavigation';
import { Layout, SettingsContainer } from '../Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from '../NavigationBar';
import CombinedLocationList, { CombinedLocationListProps } from './CombinedLocationList';
import { useRelayListContext } from './RelayListContext';
import { ScopeBarItem } from './ScopeBar';
import { useScrollPositionContext } from './ScrollPositionContext';
import {
  useOnSelectBridgeLocation,
  useOnSelectEntryLocation,
  useOnSelectExitLocation,
} from './select-location-hooks';
import {
  LocationSelectionType,
  LocationType,
  SpecialBridgeLocationType,
  SpecialLocation,
  SpecialLocationIcon,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';
import {
  StyledClearFilterButton,
  StyledContent,
  StyledFilter,
  StyledFilterIconButton,
  StyledFilterRow,
  StyledHeaderSubTitle,
  StyledNavigationBarAttachment,
  StyledNoResult,
  StyledNoResultText,
  StyledScopeBar,
  StyledSearchBar,
} from './SelectLocationStyles';
import { SpacePreAllocationView } from './SpacePreAllocationView';

export default function SelectLocation() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();
  const {
    saveScrollPosition,
    resetScrollPositions,
    scrollViewRef,
    spacePreAllocationViewRef,
  } = useScrollPositionContext();
  const { locationType, setLocationType, setSearchTerm } = useSelectLocationContext();
  const { expandSearchResults } = useRelayListContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];

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
    await updateRelaySettings({ normal: { providers: [] } });
  }, [resetScrollPositions]);

  const onClearOwnership = useCallback(async () => {
    resetScrollPositions();
    await updateRelaySettings({ normal: { ownership: Ownership.any } });
  }, [resetScrollPositions]);

  const changeLocationType = useCallback(
    (locationType: LocationType) => {
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
  const showFilters = showOwnershipFilter || showProvidersFilter;
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

                <StyledFilterIconButton
                  onClick={onViewFilter}
                  aria-label={messages.gettext('Filter')}>
                  <ImageView
                    source="icon-filter-round"
                    tintColor={colors.white40}
                    tintHoverColor={colors.white60}
                    height={24}
                    width={24}
                  />
                </StyledFilterIconButton>
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
                        { numberOfProviders: providers.length },
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
                </StyledFilterRow>
              )}

              <StyledSearchBar searchTerm={searchValue} onSearch={updateSearchTerm} />
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
  const { selectedLocationRef, spacePreAllocationViewRef } = useScrollPositionContext();
  const { relayList, expandLocation, collapseLocation, onBeforeExpand } = useRelayListContext();
  const onSelectExitLocation = useOnSelectExitLocation();
  const onSelectEntryLocation = useOnSelectEntryLocation();
  const onSelectBridgeLocation = useOnSelectBridgeLocation();

  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();

  const resetHeight = useCallback(() => spacePreAllocationViewRef.current?.reset(), []);

  if (locationType === LocationType.exit) {
    // Add "Custom" item if a custom relay is selected
    const specialList: Array<SpecialLocation<undefined>> =
      relaySettings === undefined
        ? [
            {
              type: LocationSelectionType.special,
              label: messages.gettext('Custom'),
              value: undefined,
              selected: true,
            },
          ]
        : [];

    const relayListWithSpecial = [...filterSpecialLocations(searchTerm, specialList), ...relayList];
    return (
      <LocationList
        key={locationType}
        source={relayListWithSpecial}
        selectedElementRef={selectedLocationRef}
        onSelect={onSelectExitLocation}
        onExpand={expandLocation}
        onCollapse={collapseLocation}
        onWillExpand={onBeforeExpand}
        onTransitionEnd={resetHeight}
      />
    );
  } else if (relaySettings?.tunnelProtocol !== 'openvpn') {
    return (
      <LocationList
        key={locationType}
        source={relayList}
        selectedElementRef={selectedLocationRef}
        onSelect={onSelectEntryLocation}
        onExpand={expandLocation}
        onCollapse={collapseLocation}
        onWillExpand={onBeforeExpand}
        onTransitionEnd={resetHeight}
      />
    );
  } else {
    // Add the "Automatic" item
    const specialList: Array<SpecialLocation<SpecialBridgeLocationType>> = [
      {
        type: LocationSelectionType.special,
        label: messages.gettext('Automatic'),
        icon: SpecialLocationIcon.geoLocation,
        info: messages.pgettext(
          'select-location-view',
          'The app selects a random bridge server, but servers have a higher probability the closer they are to you.',
        ),
        value: SpecialBridgeLocationType.closestToExit,
        selected: bridgeSettings?.location === 'any',
      },
    ];

    const relayListWithSpecial = [...filterSpecialLocations(searchTerm, specialList), ...relayList];
    return (
      <LocationList
        key={locationType}
        source={relayListWithSpecial}
        selectedElementRef={selectedLocationRef}
        onSelect={onSelectBridgeLocation}
        onExpand={expandLocation}
        onCollapse={collapseLocation}
        onWillExpand={onBeforeExpand}
        onTransitionEnd={resetHeight}
      />
    );
  }
}

function LocationList<T>(props: CombinedLocationListProps<T>) {
  const { searchTerm } = useSelectLocationContext();

  if (searchTerm !== '' && props.source.length === 0) {
    return (
      <StyledNoResult>
        <StyledNoResultText>
          {formatHtml(
            sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), { searchTerm }),
          )}
        </StyledNoResultText>
        <StyledNoResultText>{messages.gettext('Try a different search.')}</StyledNoResultText>
      </StyledNoResult>
    );
  } else {
    return <CombinedLocationList {...props} />;
  }
}
