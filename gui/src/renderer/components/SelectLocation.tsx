import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, View } from 'reactxp';
import { colors } from '../../config.json';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import { LocationScope } from '../redux/userinterface/reducers';
import * as Cell from './Cell';
import CustomScrollbars from './CustomScrollbars';
import { Container, Layout } from './Layout';
import LocationList from './LocationList';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  ScopeBar,
  ScopeBarItem,
  StickyContentContainer,
  StickyContentHolder,
  TitleBarItem,
} from './NavigationBar';
import styles from './SelectLocationStyles';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

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

export default class SelectLocation extends Component<IProps> {
  private scrollView = React.createRef<CustomScrollbars>();
  private exitLocationList = React.createRef<LocationList>();
  private bridgeLocationList = React.createRef<LocationList>();

  public componentDidMount() {
    this.scrollToSelectedCell();
  }

  public componentDidUpdate(prevProps: IProps) {
    if (this.props.locationScope !== prevProps.locationScope) {
      this.scrollToSelectedCell();
    }
  }

  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.select_location}>
            <NavigationContainer>
              <NavigationBar>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {// TRANSLATORS: Title label in navigation bar
                  messages.pgettext('select-location-nav', 'Select location')}
                </TitleBarItem>
              </NavigationBar>
              <StickyContentContainer style={styles.container}>
                <NavigationScrollbars ref={this.scrollView}>
                  <View style={styles.content}>
                    <SettingsHeader
                      style={this.props.allowBridgeSelection ? styles.headerWithScope : undefined}>
                      <HeaderTitle>
                        {messages.pgettext('select-location-view', 'Select location')}
                      </HeaderTitle>
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
                    </SettingsHeader>

                    {this.props.allowBridgeSelection && (
                      <StickyContentHolder style={styles.stickyHolder}>
                        <View style={styles.stickyContent}>
                          <ScopeBar
                            defaultSelectedIndex={this.props.locationScope}
                            onChange={this.props.onChangeLocationScope}>
                            <ScopeBarItem>
                              {messages.pgettext('select-location-nav', 'Entry')}
                            </ScopeBarItem>
                            <ScopeBarItem>
                              {messages.pgettext('select-location-nav', 'Exit')}
                            </ScopeBarItem>
                          </ScopeBar>
                        </View>
                      </StickyContentHolder>
                    )}

                    {this.props.locationScope === LocationScope.relay ? (
                      <LocationList
                        key={'exit-locations'}
                        ref={this.exitLocationList}
                        selectedLocation={this.props.selectedExitLocation}
                        relayLocations={this.props.relayLocations}
                        onSelect={this.props.onSelectExitLocation}
                      />
                    ) : (
                      <React.Fragment>
                        <View>
                          <ClosestToExitCell
                            onSelect={this.props.onSelectClosestToExit}
                            isSelected={this.props.selectedBridgeLocation === 'any'}
                          />
                        </View>
                        <LocationList
                          key={'bridge-locations'}
                          ref={this.bridgeLocationList}
                          selectedLocation={
                            this.props.selectedBridgeLocation !== 'any'
                              ? this.props.selectedBridgeLocation
                              : undefined
                          }
                          relayLocations={this.props.bridgeLocations}
                          onSelect={this.props.onSelectBridgeLocation}
                        />
                      </React.Fragment>
                    )}
                  </View>
                </NavigationScrollbars>
              </StickyContentContainer>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private scrollToSelectedCell() {
    const ref =
      this.props.locationScope === LocationScope.relay
        ? this.exitLocationList
        : this.bridgeLocationList;
    const locationList = ref.current;

    if (locationList) {
      const cell = locationList.selectedCell.current;
      const scrollView = this.scrollView.current;

      if (scrollView) {
        if (cell) {
          const cellDOMNode = ReactDOM.findDOMNode(cell as Element);
          if (cellDOMNode instanceof HTMLElement) {
            scrollView.scrollToElement(cellDOMNode, 'middle');
          }
        } else {
          scrollView.scrollToTop();
        }
      }
    }
  }
}

interface IClosestToExitCellProps {
  isSelected: boolean;
  onSelect: () => void;
}

function ClosestToExitCell(props: IClosestToExitCellProps) {
  return (
    <Cell.CellButton
      style={props.isSelected ? styles.selectedCell : undefined}
      cellHoverStyle={props.isSelected ? styles.selectedCell : undefined}
      onPress={props.onSelect}>
      <Cell.Icon
        source={props.isSelected ? 'icon-tick' : 'icon-nearest'}
        tintColor={colors.white}
        height={24}
        width={24}
      />
      <Cell.Label>{messages.pgettext('select-location-view', 'Closest to exit server')}</Cell.Label>
    </Cell.CellButton>
  );
}
