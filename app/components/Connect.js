// @flow

import moment from 'moment';
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import { BackendError } from '../lib/backend';

import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import ChevronRightSVG from '../assets/images/icon-chevron.svg';

import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/ipc-facade';

export type ConnectProps = {
  accountExpiry: string,
  connection: ConnectionReduxState,
  settings: SettingsReduxState,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: () => void,
  onCopyIP: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
};


export default class Connect extends Component {
  props: ConnectProps;
  state = {
    isFirstPass: true,
    showCopyIPMessage: false
  };

  _copyTimer: ?number;

  componentDidMount() {
    this.setState({ isFirstPass: false });
  }

  componentWillUnmount() {
    if(this._copyTimer) {
      clearTimeout(this._copyTimer);
      this._copyTimer = null;
    }

    this.setState({
      isFirstPass: true,
      showCopyIPMessage: false
    });
  }

  render(): React.Element<*> {
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

  renderError(error: BackendError): React.Element<*> {
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

  _findRelayName(relay: RelayLocation): ?string {
    const countries = this.props.settings.relayLocations;
    const countryPredicate = (countryCode) => (country) => country.code === countryCode;

    if(relay.country) {
      const country = countries.find(countryPredicate(relay.country));
      if(country) {
        return country.name;
      }
    } else if(relay.city) {
      const [countryCode, cityCode] = relay.city;
      const country = countries.find(countryPredicate(countryCode));
      if(country) {
        const city = country.cities.find((city) => city.code === cityCode);
        if(city) {
          return city.name;
        }
      }
    }
    return null;
  }

  _getLocationName(): string {
    const { relaySettings } = this.props.settings;
    if(relaySettings.normal) {
      const location = relaySettings.normal.location;
      if(location === 'any') {
        return 'Automatic';
      } else {
        return this._findRelayName(location) || 'Unknown';
      }
    } else if(relaySettings.custom_tunnel_endpoint) {
      return 'Custom';
    } else {
      throw new Error('Unsupported relay settings.');
    }
  }

  renderMap(): React.Element<*> {
    let [ isConnecting, isConnected, isDisconnected ] = [false, false, false];
    switch(this.props.connection.status) {
    case 'connecting': isConnecting = true; break;
    case 'connected': isConnected = true; break;
    case 'disconnected': isDisconnected = true; break;
    }

    // We decided to not include the map in the first beta release to customers
    // but it MUST be included in the following releases. Therefore we choose
    // to just comment it out
    const map = undefined;
    /*
    const altitude = (isConnecting ? 300 : 100) * 1000;
    const { location } = this.props.connection;
    const map = <Map animate={ !this.state.isFirstPass }
        location={ location || [0, 0] }
        altitude= { altitude }
        markerImagePath= { isConnected
          ? './assets/images/location-marker-secure.svg'
          : './assets/images/location-marker-unsecure.svg' } />
    */

    return (
      <div className="connect">
        <div className="connect__map">
          { map }
        </div>
        <div className="connect__container">

          { this._renderIsBlockingInternetMessage() }
          <div className="connect__status">
            { /* show spinner when connecting */ }
            <div className={ this.spinnerClass() }>
              <img src="./assets/images/icon-spinner.svg" alt="" />
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
                  <div className="connect__server-label">{ this._getLocationName() }</div>
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
