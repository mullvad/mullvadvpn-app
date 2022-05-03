import React from 'react';
import { sprintf } from 'sprintf-js';

import { colors } from '../../config.json';
import { LiftedConstraint, RelayLocation, TunnelProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import BridgeLocations, { SpecialBridgeLocationType } from './BridgeLocations';
import { CustomScrollbarsRef } from './CustomScrollbars';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Layout } from './Layout';
import LocationList, {
  DisabledReason,
  LocationSelection,
  LocationSelectionType,
} from './LocationList';
import { EntryLocations, ExitLocations } from './Locations';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import { ScopeBarItem } from './ScopeBar';
import {
  StyledClearProvidersButton,
  StyledContainer,
  StyledContent,
  StyledFilterIconButton,
  StyledNavigationBarAttachment,
  StyledProviderCountRow,
  StyledProvidersCount,
  StyledScopeBar,
  StyledSettingsHeader,
} from './SelectLocationStyles';
import { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

interface IProps {
  locale: string;
  selectedExitLocation?: RelayLocation;
  selectedEntryLocation?: RelayLocation;
  selectedBridgeLocation?: LiftedConstraint<RelayLocation>;
  relayLocations: IRelayLocationRedux[];
  bridgeLocations: IRelayLocationRedux[];
  allowEntrySelection: boolean;
  tunnelProtocol: LiftedConstraint<TunnelProtocol>;
  providers: string[];
  onClose: () => void;
  onViewFilter: () => void;
  onSelectExitLocation: (location: RelayLocation) => void;
  onSelectEntryLocation: (location: RelayLocation) => void;
  onSelectBridgeLocation: (location: RelayLocation) => void;
  onSelectClosestToExit: () => void;
  onClearProviders: () => void;
}

enum LocationScope {
  entry = 0,
  exit,
}

interface IState {
  headingHeight: number;
  locationScope: LocationScope;
}

interface ISelectLocationSnapshot {
  scrollPosition: [number, number];
  expandedLocations: RelayLocation[];
}

export default class SelectLocation extends React.Component<IProps, IState> {
  public state = { headingHeight: 0, locationScope: LocationScope.exit };

  private scrollView = React.createRef<CustomScrollbarsRef>();
  private spacePreAllocationViewRef = React.createRef<SpacePreAllocationView>();
  private selectedExitLocationRef = React.createRef<React.ReactInstance>();
  private selectedEntryLocationRef = React.createRef<React.ReactInstance>();
  private selectedBridgeLocationRef = React.createRef<React.ReactInstance>();

  private exitLocationList = React.createRef<LocationList<never>>();
  private entryLocationList = React.createRef<LocationList<never>>();
  private bridgeLocationList = React.createRef<LocationList<SpecialBridgeLocationType>>();

  private snapshotByScope: Partial<Record<LocationScope, ISelectLocationSnapshot>> = {};

  private headerRef = React.createRef<HTMLHeadingElement>();

  public componentDidMount() {
    this.scrollToSelectedCell();
    this.setState((state) => ({
      headingHeight: this.headerRef.current?.offsetHeight ?? state.headingHeight,
    }));
  }

  public componentDidUpdate(
    _prevProps: IProps,
    prevState: IState,
    snapshot?: ISelectLocationSnapshot,
  ) {
    if (this.state.locationScope !== prevState.locationScope) {
      this.restoreScrollPosition(this.state.locationScope);

      if (snapshot) {
        this.snapshotByScope[prevState.locationScope] = snapshot;
      }
    }
  }

  public getSnapshotBeforeUpdate(
    prevProps: IProps,
    prevState: IState,
  ): ISelectLocationSnapshot | undefined {
    const scrollView = this.scrollView.current;
    const locationList = this.getLocationListRef(prevProps, prevState);

    if (scrollView && locationList) {
      return {
        scrollPosition: scrollView.getScrollPosition(),
        expandedLocations: locationList.getExpandedLocations(),
      };
    } else {
      return undefined;
    }
  }

  public render() {
    return (
      <BackAction icon="close" action={this.props.onClose}>
        <Layout>
          <StyledContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('select-location-nav', 'Select location')
                    }
                  </TitleBarItem>

                  <StyledFilterIconButton
                    onClick={this.props.onViewFilter}
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
              <NavigationScrollbars ref={this.scrollView}>
                <SpacePreAllocationView ref={this.spacePreAllocationViewRef}>
                  <StyledNavigationBarAttachment top={-this.state.headingHeight}>
                    <StyledSettingsHeader ref={this.headerRef}>
                      <HeaderTitle>
                        {
                          // TRANSLATORS: Heading in select location view
                          messages.pgettext('select-location-view', 'Select location')
                        }
                      </HeaderTitle>
                      {this.renderHeaderSubtitle()}
                    </StyledSettingsHeader>

                    {this.props.providers.length > 0 && (
                      <StyledProviderCountRow>
                        {messages.pgettext('select-location-view', 'Filtered:')}
                        <StyledProvidersCount>
                          {sprintf(
                            messages.pgettext(
                              'select-location-view',
                              'Providers: %(numberOfProviders)d',
                            ),
                            {
                              numberOfProviders: this.props.providers.length,
                            },
                          )}
                          <StyledClearProvidersButton
                            aria-label={messages.gettext('Clear')}
                            onClick={this.props.onClearProviders}>
                            <ImageView
                              height={16}
                              width={16}
                              source="icon-close"
                              tintColor={colors.white60}
                              tintHoverColor={colors.white80}
                            />
                          </StyledClearProvidersButton>
                        </StyledProvidersCount>
                      </StyledProviderCountRow>
                    )}
                    {this.props.allowEntrySelection && (
                      <StyledScopeBar
                        defaultSelectedIndex={this.state.locationScope}
                        onChange={this.onChangeLocationScope}>
                        <ScopeBarItem>
                          {messages.pgettext('select-location-view', 'Entry')}
                        </ScopeBarItem>
                        <ScopeBarItem>
                          {messages.pgettext('select-location-view', 'Exit')}
                        </ScopeBarItem>
                      </StyledScopeBar>
                    )}
                  </StyledNavigationBarAttachment>

                  <StyledContent>{this.renderLocationList()}</StyledContent>
                </SpacePreAllocationView>
              </NavigationScrollbars>
            </NavigationContainer>
          </StyledContainer>
        </Layout>
      </BackAction>
    );
  }

  public restoreScrollPosition(scope: LocationScope) {
    const snapshot = this.snapshotByScope[scope];

    if (snapshot) {
      this.scrollToPosition(...snapshot.scrollPosition);
    } else {
      this.scrollToSelectedCell();
    }
  }

  private getLocationListRef(prevProps: IProps, prevState: IState) {
    if (prevState.locationScope === LocationScope.exit) {
      return this.exitLocationList.current;
    } else if (prevProps.tunnelProtocol === 'wireguard') {
      return this.entryLocationList.current;
    } else {
      return this.bridgeLocationList.current;
    }
  }

  private getSelectedLocationRef() {
    if (this.state.locationScope === LocationScope.exit) {
      return this.selectedExitLocationRef.current;
    } else if (this.props.tunnelProtocol === 'wireguard') {
      return this.selectedEntryLocationRef.current;
    } else {
      return this.selectedBridgeLocationRef.current;
    }
  }

  private renderHeaderSubtitle() {
    if (this.props.allowEntrySelection) {
      if (this.props.tunnelProtocol === 'openvpn') {
        return (
          <HeaderSubTitle>
            {messages.pgettext(
              'select-location-view',
              'While connected, your traffic will be routed through two secure locations, the entry point (a bridge server) and the exit point (a VPN server).',
            )}
          </HeaderSubTitle>
        );
      } else {
        return (
          <HeaderSubTitle>
            {messages.pgettext(
              'select-location-view',
              'While connected, your traffic will be routed through two secure locations, the entry point and the exit point (needs to be two different VPN servers).',
            )}
          </HeaderSubTitle>
        );
      }
    } else {
      return null;
    }
  }

  private renderLocationList() {
    if (this.state.locationScope === LocationScope.exit) {
      const disabledLocation = this.props.selectedEntryLocation
        ? {
            location: this.props.selectedEntryLocation,
            reason: DisabledReason.entry,
          }
        : undefined;
      return (
        <ExitLocations
          ref={this.exitLocationList}
          source={this.props.relayLocations}
          locale={this.props.locale}
          defaultExpandedLocations={this.getExpandedLocationsFromSnapshot()}
          selectedValue={this.props.selectedExitLocation}
          selectedElementRef={this.selectedExitLocationRef}
          disabledLocation={disabledLocation}
          onSelect={this.onSelectExitLocation}
          onWillExpand={this.onWillExpand}
          onTransitionEnd={this.resetHeight}
        />
      );
    } else if (this.props.tunnelProtocol === 'any' || this.props.tunnelProtocol === 'wireguard') {
      const disabledLocation = this.props.selectedExitLocation
        ? {
            location: this.props.selectedExitLocation,
            reason: DisabledReason.exit,
          }
        : undefined;
      return (
        <EntryLocations
          ref={this.entryLocationList}
          source={this.props.relayLocations}
          locale={this.props.locale}
          defaultExpandedLocations={this.getExpandedLocationsFromSnapshot()}
          selectedValue={this.props.selectedEntryLocation}
          selectedElementRef={this.selectedEntryLocationRef}
          disabledLocation={disabledLocation}
          onSelect={this.onSelectEntryLocation}
          onWillExpand={this.onWillExpand}
          onTransitionEnd={this.resetHeight}
        />
      );
    } else {
      return (
        <BridgeLocations
          ref={this.bridgeLocationList}
          source={this.props.bridgeLocations}
          locale={this.props.locale}
          defaultExpandedLocations={this.getExpandedLocationsFromSnapshot()}
          selectedValue={this.props.selectedBridgeLocation}
          selectedElementRef={this.selectedBridgeLocationRef}
          onSelect={this.onSelectBridgeLocation}
          onWillExpand={this.onWillExpand}
          onTransitionEnd={this.resetHeight}
        />
      );
    }
  }

  private resetHeight = () => {
    this.spacePreAllocationViewRef.current?.reset();
  };

  private getExpandedLocationsFromSnapshot(): RelayLocation[] | undefined {
    const snapshot = this.snapshotByScope[this.state.locationScope];
    if (snapshot) {
      return snapshot.expandedLocations;
    } else {
      return undefined;
    }
  }

  private scrollToPosition(x: number, y: number) {
    const scrollView = this.scrollView.current;
    if (scrollView) {
      scrollView.scrollTo(x, y);
    }
  }

  private scrollToSelectedCell() {
    const ref = this.getSelectedLocationRef();
    const scrollView = this.scrollView.current;

    if (scrollView) {
      if (ref) {
        if (ref instanceof HTMLElement) {
          scrollView.scrollToElement(ref, 'middle');
        }
      } else {
        scrollView.scrollToTop();
      }
    }
  }

  private onChangeLocationScope = (locationScope: LocationScope) => {
    this.setState({ locationScope });
  };

  private onSelectExitLocation = (location: LocationSelection<never>) => {
    if (location.type === LocationSelectionType.relay) {
      this.props.onSelectExitLocation(location.value);
    }
  };

  private onSelectEntryLocation = (location: LocationSelection<never>) => {
    this.props.onSelectEntryLocation(location.value);
  };

  private onSelectBridgeLocation = (location: LocationSelection<SpecialBridgeLocationType>) => {
    if (location.type === LocationSelectionType.relay) {
      this.props.onSelectBridgeLocation(location.value);
    } else if (
      location.type === LocationSelectionType.special &&
      location.value === SpecialBridgeLocationType.closestToExit
    ) {
      this.props.onSelectClosestToExit();
    }
  };

  private onWillExpand = (locationRect: DOMRect, expandedContentHeight: number) => {
    locationRect.height += expandedContentHeight;
    this.spacePreAllocationViewRef.current?.allocate(expandedContentHeight);
    this.scrollView.current?.scrollIntoView(locationRect);
  };
}

interface ISpacePreAllocationView {
  children?: React.ReactNode;
}

class SpacePreAllocationView extends React.Component<ISpacePreAllocationView> {
  private ref = React.createRef<HTMLDivElement>();

  public allocate(height: number) {
    if (this.ref.current) {
      this.minHeight = this.ref.current.offsetHeight + height + 'px';
    }
  }

  public reset = () => {
    this.minHeight = 'auto';
  };

  public render() {
    return <div ref={this.ref}>{this.props.children}</div>;
  }

  private set minHeight(value: string) {
    const element = this.ref.current;
    if (element) {
      element.style.minHeight = value;
    }
  }
}
