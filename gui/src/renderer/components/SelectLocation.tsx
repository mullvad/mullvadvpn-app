import React from 'react';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import { LocationScope } from '../redux/userinterface/reducers';
import BridgeLocations, { SpecialBridgeLocationType } from './BridgeLocations';
import CustomScrollbars from './CustomScrollbars';
import ExitLocations from './ExitLocations';
import ImageView from './ImageView';
import { Layout } from './Layout';
import LocationList, { LocationSelection, LocationSelectionType } from './LocationList';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import { ScopeBarItem } from './ScopeBar';
import {
  StyledContainer,
  StyledContent,
  StyledFilterIconButton,
  StyledFilterContainer,
  StyledFilterMenu,
  StyledNavigationBarAttachment,
  StyledScopeBar,
  StyledFilterByProviderButton,
  StyledProvidersCount,
  StyledProviderCountRow,
  StyledClearProvidersButton,
  StyledSettingsHeader,
} from './SelectLocationStyles';
import { HeaderTitle } from './SettingsHeader';

interface IProps {
  locationScope: LocationScope;
  selectedExitLocation?: RelayLocation;
  selectedBridgeLocation?: LiftedConstraint<RelayLocation>;
  relayLocations: IRelayLocationRedux[];
  bridgeLocations: IRelayLocationRedux[];
  allowBridgeSelection: boolean;
  providers: string[];
  onClose: () => void;
  onViewFilterByProvider: () => void;
  onChangeLocationScope: (location: LocationScope) => void;
  onSelectExitLocation: (location: RelayLocation) => void;
  onSelectBridgeLocation: (location: RelayLocation) => void;
  onSelectClosestToExit: () => void;
  onClearProviders: () => void;
}

interface IState {
  showFilterMenu: boolean;
  headingHeight: number;
}

interface ISelectLocationSnapshot {
  scrollPosition: [number, number];
  expandedLocations: RelayLocation[];
}

export default class SelectLocation extends React.Component<IProps, IState> {
  // The default headingHeight value is based on a one-line heading.
  public state = { showFilterMenu: false, headingHeight: 50 };

  private scrollView = React.createRef<CustomScrollbars>();
  private spacePreAllocationViewRef = React.createRef<SpacePreAllocationView>();
  private selectedExitLocationRef = React.createRef<React.ReactInstance>();
  private selectedBridgeLocationRef = React.createRef<React.ReactInstance>();

  private exitLocationList = React.createRef<LocationList<never>>();
  private bridgeLocationList = React.createRef<LocationList<SpecialBridgeLocationType>>();

  private snapshotByScope: { [index: number]: ISelectLocationSnapshot } = {};

  private filterButtonRef = React.createRef<HTMLDivElement>();
  private headingRef = React.createRef<HTMLHeadingElement>();

  public componentDidMount() {
    this.scrollToSelectedCell();
    this.setState((state) => ({
      // 10 px is the margin ontop of the heading.
      headingHeight: (this.headingRef.current?.offsetHeight ?? state.headingHeight) + 10,
    }));
  }

  public componentDidUpdate(
    prevProps: IProps,
    _prevState: unknown,
    snapshot?: ISelectLocationSnapshot,
  ) {
    if (this.props.locationScope !== prevProps.locationScope) {
      this.restoreScrollPosition(this.props.locationScope);

      if (snapshot) {
        this.snapshotByScope[prevProps.locationScope] = snapshot;
      }
    }
  }

  public getSnapshotBeforeUpdate(prevProps: IProps): ISelectLocationSnapshot | undefined {
    const scrollView = this.scrollView.current;
    const locationList =
      prevProps.locationScope === LocationScope.relay
        ? this.exitLocationList.current
        : this.bridgeLocationList.current;

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
      <Layout onClick={this.onClickAnywhere}>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('select-location-nav', 'Select location')
                  }
                </TitleBarItem>

                <StyledFilterContainer ref={this.filterButtonRef}>
                  <StyledFilterIconButton
                    onClick={this.toggleFilterMenu}
                    aria-label={messages.gettext('Filter')}>
                    <ImageView
                      source="icon-filter-round"
                      tintColor={colors.white40}
                      tintHoverColor={colors.white60}
                      height={24}
                      width={24}
                    />
                  </StyledFilterIconButton>
                  {this.state.showFilterMenu && (
                    <StyledFilterMenu>
                      <StyledFilterByProviderButton onClick={this.props.onViewFilterByProvider}>
                        {messages.pgettext('select-location-view', 'Filter by provider')}
                      </StyledFilterByProviderButton>
                    </StyledFilterMenu>
                  )}
                </StyledFilterContainer>
              </NavigationItems>
            </NavigationBar>
            <NavigationScrollbars ref={this.scrollView}>
              <SpacePreAllocationView ref={this.spacePreAllocationViewRef}>
                <StyledNavigationBarAttachment top={-this.state.headingHeight}>
                  <StyledSettingsHeader>
                    <HeaderTitle ref={this.headingRef}>
                      {
                        // TRANSLATORS: Heading in select location view
                        messages.pgettext('select-location-view', 'Select location')
                      }
                    </HeaderTitle>
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
                  {this.props.allowBridgeSelection && (
                    <StyledScopeBar
                      defaultSelectedIndex={this.props.locationScope}
                      onChange={this.props.onChangeLocationScope}>
                      <ScopeBarItem>
                        {messages.pgettext('select-location-nav', 'Entry')}
                      </ScopeBarItem>
                      <ScopeBarItem>
                        {messages.pgettext('select-location-nav', 'Exit')}
                      </ScopeBarItem>
                    </StyledScopeBar>
                  )}
                </StyledNavigationBarAttachment>

                <StyledContent>
                  {this.props.locationScope === LocationScope.relay ? (
                    <ExitLocations
                      ref={this.exitLocationList}
                      source={this.props.relayLocations}
                      defaultExpandedLocations={this.getExpandedLocationsFromSnapshot()}
                      selectedValue={this.props.selectedExitLocation}
                      selectedElementRef={this.selectedExitLocationRef}
                      onSelect={this.onSelectExitLocation}
                      onWillExpand={this.onWillExpand}
                      onTransitionEnd={this.resetHeight}
                    />
                  ) : (
                    <BridgeLocations
                      ref={this.bridgeLocationList}
                      source={this.props.bridgeLocations}
                      defaultExpandedLocations={this.getExpandedLocationsFromSnapshot()}
                      selectedValue={this.props.selectedBridgeLocation}
                      selectedElementRef={this.selectedBridgeLocationRef}
                      onSelect={this.onSelectBridgeLocation}
                      onWillExpand={this.onWillExpand}
                      onTransitionEnd={this.resetHeight}
                    />
                  )}
                </StyledContent>
              </SpacePreAllocationView>
            </NavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
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

  private resetHeight = () => {
    this.spacePreAllocationViewRef.current?.reset();
  };

  private getExpandedLocationsFromSnapshot(): RelayLocation[] | undefined {
    const snapshot = this.snapshotByScope[this.props.locationScope];
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
    const ref =
      this.props.locationScope === LocationScope.relay
        ? this.selectedExitLocationRef.current
        : this.selectedBridgeLocationRef.current;
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

  private onSelectExitLocation = (location: LocationSelection<never>) => {
    if (location.type === LocationSelectionType.relay) {
      this.props.onSelectExitLocation(location.value);
    }
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

  private toggleFilterMenu = () => {
    this.setState((state) => ({
      showFilterMenu: !state.showFilterMenu,
    }));
  };

  private onClickAnywhere = (event: React.MouseEvent<HTMLDivElement>) => {
    if (
      this.state.showFilterMenu &&
      !this.filterButtonRef.current?.contains(event.target as HTMLElement)
    ) {
      this.setState({ showFilterMenu: false });
    }
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
