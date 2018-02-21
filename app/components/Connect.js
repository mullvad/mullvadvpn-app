// @flow

import moment from 'moment';
import * as React from 'react';
import { Layout, Container, Header } from './Layout';
import { BackendError } from '../lib/backend';
import Map from './Map';

import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import ChevronRightSVG from '../assets/images/icon-chevron.svg';

import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';

export type ConnectProps = {
  connection: ConnectionReduxState,
  accountExpiry: string,
  selectedRelayName: string,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onCopyIP: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
};

type ConnectState = {
  showCopyIPMessage: boolean,
  mapOffset: [number, number],
};

export default class Connect extends React.Component<ConnectProps, ConnectState> {
  state = {
    showCopyIPMessage: false,
    mapOffset: [0, 0],
  };

  _copyTimer: ?TimeoutID;

  shouldComponentUpdate(nextProps: ConnectProps, nextState: ConnectState) {
    const { connection: prevConnection, ...otherPrevProps } = this.props;
    const { connection: nextConnection, ...otherNextProps } = nextProps;

    const prevState = this.state;

    return (
      // shallow compare the connection
      !shallowCompare(prevConnection, nextConnection) ||
      !shallowCompare(otherPrevProps, otherNextProps) ||

      prevState.mapOffset[0] !== nextState.mapOffset[0] ||
      prevState.mapOffset[1] !== nextState.mapOffset[1] ||
      prevState.showCopyIPMessage !== nextState.showCopyIPMessage
    );
  }

  componentWillUnmount() {
    if(this._copyTimer) {
      clearTimeout(this._copyTimer);
    }
  }

  render() {
    const error = this.displayError();
    const child = error ? this.renderError(error) : this.renderMap();

    return (
      <Layout>
        <Header style={ this.headerStyle() } showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          { child }
        </Container>
      </Layout>
    );
  }

  renderError(error: BackendError) {
    return (
      <div className="connect">
        <div className="connect__status">
          <div className="connect__status-icon">
            <img src="./assets/images/icon-fail.svg" alt="" />
          </div>
          <div className="connect__error-title">
            { error.title }
          </div>
          <div className="connect__error-message">
            { error.message }
          </div>
          { error.type === 'NO_CREDIT' ?
            <div>
              <button className="button button--positive" onClick={ this.onExternalLink.bind(this, 'purchase') }>
                <span className="button-label">Buy more time</span>
                <ExternalLinkSVG className="button-icon button-icon--16" />
              </button>
            </div>
            : null
          }
        </div>
      </div>
    );
  }

  _getMapProps() {
    const { longitude, latitude, status } = this.props.connection;

    // when the user location is known
    if(typeof(longitude) === 'number' && typeof(latitude) === 'number') {
      return {
        center: [longitude, latitude],
        // do not show the marker when connecting
        showMarker: status !== 'connecting',
        markerStyle: status === 'connected' ? 'secure' : 'unsecure',
        // zoom in when connected
        zoomLevel: status === 'connected' ? 'low' : 'medium',
        // a magic offset to align marker with spinner
        offset: [0, 123],
      };
    } else {
      return {
        center: [0, 0],
        showMarker: false,
        markerStyle: 'unsecure',
        // show the world when user location is not known
        zoomLevel: 'high',
        // remove the offset since the marker is hidden
        offset: [0, 0],
      };
    }
  }

  _updateMapOffset = (spinnerNode: ?HTMLElement) => {
    if(spinnerNode) {
      // calculate the vertical offset from the center of the map
      // to shift the center of the map upwards to align the centers
      // of spinner and marker on the map
      const y = spinnerNode.offsetTop + spinnerNode.clientHeight * 0.5;
      this.setState({
        mapOffset: [0, y]
      });
    }
  }

  renderMap() {
    let [ isConnecting, isConnected, isDisconnected ] = [false, false, false];
    switch(this.props.connection.status) {
    case 'connecting': isConnecting = true; break;
    case 'connected': isConnected = true; break;
    case 'disconnected': isDisconnected = true; break;
    }

    return (
      <div className="connect">
        <div className="connect__map">
          <Map style={{ width: '100%', height: '100%' }} { ...this._getMapProps() } />
        </div>
        <div className="connect__container">

          { this._renderIsBlockingInternetMessage() }
          <div className="connect__status">
            { /* show spinner when connecting */ }
            <div className={ this.spinnerClass() }>
              <img src="./assets/images/icon-spinner.svg" alt="" ref={ this._updateMapOffset } />
            </div>

            <div className={ this.networkSecurityClass() }>{ this.networkSecurityMessage() }</div>

            { /*
              **********************************
              Begin: Location block
              **********************************
            */ }

            { /* location when connecting or disconnected */ }
            { isConnecting || isDisconnected ?
              <div className="connect__status-location">
                <span>{ this.props.connection.country }</span>
              </div>
              : null
            }

            { /* location when connected */ }
            { isConnected ?
              <div className="connect__status-location">
                { this.props.connection.city }
                { this.props.connection.city && <br/> }
                { this.props.connection.country }
              </div>
              :null
            }

            { /*
              **********************************
              End: Location block
              **********************************
            */ }

            <div className={ this.ipAddressClass() } onClick={ this.onIPAddressClick.bind(this) }>
              { (isConnected || isDisconnected) ? (
                <span>{
                  this.state.showCopyIPMessage ?
                    'IP copied to clipboard!' :
                    this.props.connection.ip
                }</span>) : null }
            </div>
          </div>


          { /*
            **********************************
            Begin: Footer block
            **********************************
          */ }

          { /* footer when disconnected */ }
          { isDisconnected ?
            <div className="connect__footer">
              <div className="connect__row">
                <button className="connect__server button button--neutral button--blur" onClick={ this.props.onSelectLocation }>
                  <div className="connect__server-label">{ this.props.selectedRelayName }</div>
                  <div className="connect__server-chevron"><ChevronRightSVG /></div>
                </button>
              </div>

              <div className="connect__row">
                <button className="button button--positive" onClick={ this.props.onConnect }>Secure my connection</button>
              </div>
            </div>
            : null
          }

          { /* footer when connecting */ }
          { isConnecting ?
            <div className="connect__footer">
              <div className="connect__row">
                <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
              </div>

              <div className="connect__row">
                <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Cancel</button>
              </div>
            </div>
            : null
          }

          { /* footer when connected */ }
          { isConnected ?
            <div className="connect__footer">
              <div className="connect__row">
                <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
              </div>

              <div className="connect__row">
                <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Disconnect</button>
              </div>
            </div>
            : null
          }

          { /*
            **********************************
            End: Footer block
            **********************************
          */ }

        </div>
      </div>
    );
  }

  _renderIsBlockingInternetMessage() {
    let animationClass = 'hide';
    if (this.props.connection.status === 'connecting') {
      animationClass = 'show';
    }

    return <div className={`connect__blocking-container ${animationClass}`}>
      <div className="connect__blocking-message">
        <div className="connect__blocking-icon">&nbsp;</div>
        blocking internet
      </div>
    </div>;
  }

  // Handlers

  onExternalLink(type: string) {
    this.props.onExternalLink(type);
  }

  onIPAddressClick() {
    this._copyTimer && clearTimeout(this._copyTimer);
    this._copyTimer = setTimeout(() => this.setState({ showCopyIPMessage: false }), 3000);
    this.setState({ showCopyIPMessage: true });
    this.props.onCopyIP();
  }

  // Private

  headerStyle(): HeaderBarStyle {
    switch(this.props.connection.status) {
    case 'disconnected':
      return 'error';
    case 'connecting':
    case 'connected':
      return 'success';
    }
    throw new Error('Invalid ConnectionState');
  }

  networkSecurityClass(): string {
    let classes = ['connect__status-security'];
    if(this.props.connection.status === 'connected') {
      classes.push('connect__status-security--secure');
    } else if(this.props.connection.status === 'disconnected') {
      classes.push('connect__status-security--unsecured');
    }

    return classes.join(' ');
  }

  networkSecurityMessage(): string {
    switch(this.props.connection.status) {
    case 'connected': return 'Secure connection';
    case 'connecting': return 'Creating secure connection';
    default: return 'Unsecured connection';
    }
  }

  spinnerClass(): string {
    var classes = ['connect__status-icon'];
    if(this.props.connection.status !== 'connecting') {
      classes.push('connect__status-icon--hidden');
    }
    return classes.join(' ');
  }

  ipAddressClass(): string {
    var classes = ['connect__status-ipaddress'];
    if(this.props.connection.status === 'connecting') {
      classes.push('connect__status-ipaddress--invisible');
    }
    return classes.join(' ');
  }

  displayError(): ?BackendError {
    // Offline?
    if(!this.props.connection.isOnline) {
      return new BackendError('NO_INTERNET');
    }

    // No credit?
    const expiry = this.props.accountExpiry;
    if(expiry && moment(expiry).isSameOrBefore(moment())) {
      return new BackendError('NO_CREDIT');
    }

    return null;
  }
}

function shallowCompare(lhs: Object, rhs: Object) {
  const keys = Object.keys(lhs);
  return (
    keys.length === Object.keys(rhs).length &&
    keys.every(key => lhs[key] === rhs[key])
  );
}