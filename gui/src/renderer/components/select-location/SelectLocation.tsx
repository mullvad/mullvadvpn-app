import { useCallback, useEffect } from 'react';
import { sprintf } from 'sprintf-js';

import { colors } from '../../../config.json';
import { Ownership } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useAppContext } from '../../context';
import { useHistory } from '../../lib/history';
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
import LocationList from './LocationList';
import { ScopeBar, ScopeBarItem } from './ScopeBar';
import {
  useExpandedLocations,
  useOnSelectBridgeLocation,
  useOnSelectLocation,
  useRelayList,
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
  StyledNavigationBarAttachment,
  StyledSearchBar,
} from './SelectLocationStyles';
import { SpacePreAllocationView } from './SpacePreAllocationView';

export default function SelectLocation() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();
  const { saveScrollPosition, resetScrollPositions, applyScrollPosition } = useScrollPosition();
  const { scrollViewRef, spacePreAllocationViewRef, locationType, setLocationType, searchTerm, setSearchTerm } = useSelectLocationContext();

  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];

  const onClose = useCallback(() => history.dismiss(), [history]);
  const onViewFilter = useCallback(() => history.push(RoutePath.filter), [history]);

  const tunnelProtocol = relaySettings?.tunnelProtocol ?? 'any';
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const allowEntrySelection =
    (tunnelProtocol === 'openvpn' && bridgeState === 'on') ||
    (tunnelProtocol !== 'openvpn' && relaySettings?.wireguard.useMultihop);

  const onClearProviders = useCallback(async () => {
    resetScrollPositions();
    await updateRelaySettings({ normal: { providers: [] } });
  }, []);

  const onClearOwnership = useCallback(async () => {
    resetScrollPositions();
    await updateRelaySettings({ normal: { ownership: Ownership.any } });
  }, []);

  const changeLocationType = useCallback(
    (locationType: LocationType) => {
      saveScrollPosition();
      setLocationType(locationType);
    },
    [saveScrollPosition],
  );

  const updateSearchTerm = useCallback(
    (value: string) => {
      resetScrollPositions();
      setSearchTerm(value);
    },
    [resetScrollPositions],
  );

  useEffect(applyScrollPosition, [applyScrollPosition]);

  const showOwnershipFilter = ownership !== Ownership.any;
  const showProvidersFilter = providers.length > 0;
  const showFilters = showOwnershipFilter || showProvidersFilter;
  return (
    <BackAction icon="close" action={onClose}>
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
                <ScopeBar selectedIndex={locationType} onChange={changeLocationType}>
                  <ScopeBarItem>{messages.pgettext('select-location-view', 'Entry')}</ScopeBarItem>
                  <ScopeBarItem>{messages.pgettext('select-location-view', 'Exit')}</ScopeBarItem>
                </ScopeBar>
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

              <StyledSearchBar searchTerm={searchTerm} onSearch={updateSearchTerm} />
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
  const { locationType, selectedLocationRef, spacePreAllocationViewRef } = useSelectLocationContext();
  const relayList = useRelayList();
  const { expandLocation, collapseLocation, updateExpandedLocations } = useExpandedLocations();
  const { onBeforeExpand } = useScrollPosition();
  const onSelectLocation = useOnSelectLocation();
  const onSelectBridgeLocation = useOnSelectBridgeLocation();

  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();

  const resetHeight = useCallback(() => spacePreAllocationViewRef.current?.reset(), []);

  useEffect(updateExpandedLocations, [updateExpandedLocations]);

  if (locationType === LocationType.exit) {
    return (
      <LocationList
        key={locationType}
        source={relayList}
        selectedElementRef={selectedLocationRef}
        onSelect={onSelectLocation}
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
        onSelect={onSelectLocation}
        onExpand={expandLocation}
        onCollapse={collapseLocation}
        onWillExpand={onBeforeExpand}
        onTransitionEnd={resetHeight}
      />
    );
  } else {
    const automaticItem: SpecialLocation<SpecialBridgeLocationType> = {
      type: LocationSelectionType.special,
      label: messages.gettext('Automatic'),
      icon: SpecialLocationIcon.geoLocation,
      info: messages.pgettext(
        'select-location-view',
        'The app selects a random bridge server, but servers have a higher probability the closer they are to you.',
      ),
      value: SpecialBridgeLocationType.closestToExit,
      selected: bridgeSettings?.location === 'any',
      disabled: false,
    };

    const bridgeRelayList = [automaticItem, ...relayList];
    return (
      <LocationList
        key={locationType}
        source={bridgeRelayList}
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

function useScrollPosition() {
  const { locationType, scrollPositions, scrollViewRef, spacePreAllocationViewRef, selectedLocationRef, searchTerm } = useSelectLocationContext();
  const relaySettings = useNormalRelaySettings();

  const saveScrollPosition = useCallback(() => {
    const scrollPosition = scrollViewRef.current?.getScrollPosition();
    if (scrollPositions.current && scrollPosition) {
      scrollPositions.current[locationType] = scrollPosition;
    }
  }, [locationType]);

  const resetScrollPositions = useCallback(() => {
    for (const locationTypeVariant of [LocationType.entry, LocationType.exit]) {
      if (
        scrollPositions.current &&
        (scrollPositions.current[locationTypeVariant] || locationTypeVariant === locationType)
      ) {
        scrollPositions.current[locationTypeVariant] = [0, 0];
      }
    }
  }, [locationType]);

  const applyScrollPosition = useCallback(() => {
    const scrollPosition = scrollPositions.current?.[locationType];
    if (scrollPosition) {
      scrollViewRef.current?.scrollTo(...scrollPosition);
    } else if (selectedLocationRef.current) {
      scrollViewRef.current?.scrollToElement(selectedLocationRef.current, 'middle');
    } else {
      scrollViewRef.current?.scrollToTop();
    }
  }, [locationType, searchTerm, relaySettings?.ownership, relaySettings?.providers]);

  const onBeforeExpand = useCallback((locationRect: DOMRect, expandedContentHeight: number) => {
    locationRect.height += expandedContentHeight;
    spacePreAllocationViewRef.current?.allocate(expandedContentHeight);
    scrollViewRef.current?.scrollIntoView(locationRect);
  }, []);

  return { spacePreAllocationViewRef, saveScrollPosition, resetScrollPositions, applyScrollPosition, onBeforeExpand };
}
