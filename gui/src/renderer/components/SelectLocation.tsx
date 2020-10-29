import React from 'react';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import { LocationScope } from '../redux/userinterface/reducers';
import BridgeLocations, { SpecialBridgeLocationType } from './BridgeLocations';
import CustomScrollbars from './CustomScrollbars';
import ExitLocations from './ExitLocations';
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
  StyledNavigationBarAttachment,
  StyledScopeBar,
} from './SelectLocationStyles';
import { HeaderSubTitle } from './SettingsHeader';

interface IProps {
  locationScope: LocationScope;
  selectedExitLocation?: RelayLocation;
  selectedBridgeLocation?: LiftedConstraint<RelayLocation>;
  relayLocations: IRelayLocationRedux[];
  bridgeLocations: IRelayLocationRedux[];
  allowBridgeSelection: boolean;
  onClose: () => void;
  onChangeLocationScope: (location: LocationScope) => void;
  onSelectExitLocation: (location: RelayLocation) => void;
  onSelectBridgeLocation: (location: RelayLocation) => void;
  onSelectClosestToExit: () => void;
}

interface ISelectLocationSnapshot {
  scrollPosition: [number, number];
  expandedLocations: RelayLocation[];
}

export default class SelectLocation extends React.Component<IProps> {
  private scrollView = React.createRef<CustomScrollbars>();
  private spacePreAllocationViewRef = React.createRef<SpacePreAllocationView>();
  private selectedExitLocationRef = React.createRef<React.ReactInstance>();
  private selectedBridgeLocationRef = React.createRef<React.ReactInstance>();

  private exitLocationList = React.createRef<LocationList<never>>();
  private bridgeLocationList = React.createRef<LocationList<SpecialBridgeLocationType>>();

  private snapshotByScope: { [index: number]: ISelectLocationSnapshot } = {};

  public componentDidMount() {
    this.scrollToSelectedCell();
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
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar alwaysDisplayBarTitle={true}>
              <NavigationItems>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('select-location-nav', 'Select location')
                  }
                </TitleBarItem>
              </NavigationItems>
              <StyledNavigationBarAttachment>
                <HeaderSubTitle>
                  {this.props.allowBridgeSelection
                    ? messages.pgettext(
                        'select-location-view',
                        'While connected, your traffic will be routed through two secure locations, the entry point (a bridge server) and the exit point (a VPN server).',
                      )
                    : messages.pgettext(
                        'select-location-view',
                        'While connected, your real location is masked with a private and secure location in the selected region.',
                      )}
                </HeaderSubTitle>
                {this.props.allowBridgeSelection && (
                  <StyledScopeBar
                    defaultSelectedIndex={this.props.locationScope}
                    onChange={this.props.onChangeLocationScope}>
                    <ScopeBarItem>{messages.pgettext('select-location-nav', 'Entry')}</ScopeBarItem>
                    <ScopeBarItem>{messages.pgettext('select-location-nav', 'Exit')}</ScopeBarItem>
                  </StyledScopeBar>
                )}
              </StyledNavigationBarAttachment>
            </NavigationBar>
            <NavigationScrollbars ref={this.scrollView}>
              <SpacePreAllocationView ref={this.spacePreAllocationViewRef}>
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
