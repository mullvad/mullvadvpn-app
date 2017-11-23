// @flow

import moment from 'moment';
import React, { Component } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { BackendError } from '../lib/backend';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

import type { ServerInfo } from '../lib/backend';
import type { HeaderBarStyle } from './HeaderBar';
import type { ConnectionReduxState } from '../redux/connection/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

export type ConnectProps = {
  accountExpiry: string,
  connection: ConnectionReduxState,
  settings: SettingsReduxState,
  onSettings: () => void,
  onSelectLocation: () => void,
  onConnect: (host: string) => void,
  onCopyIP: () => void,
  onDisconnect: () => void,
  onExternalLink: (type: string) => void,
  getServerInfo: (identifier: string) => ?ServerInfo
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
          <If condition={ error.type === 'NO_CREDIT' }>
            <Then>
              <div>
                <button className="button button--positive" onClick={ this.onExternalLink.bind(this, 'purchase') }>
                  <span className="button-label">Buy more time</span>
                  <ExternalLinkSVG className="button-icon button-icon--16" />
                </button>
              </div>
            </Then>
          </If>
        </div>
      </div>
    );
  }

  _getServerInfo() {
    const { relaySettings } = this.props.settings;
    if (relaySettings.host === 'any') {
      return {
        name: 'Automatic',
        country: 'Automatic',
        city: 'Automatic',
        address: '',
      };
    }

    return this.props.getServerInfo(relaySettings.host);
  }

  renderMap(): React.Element<*> {
    const serverInfo = this._getServerInfo();

    let isConnecting = false;
    let isConnected = false;
    let isDisconnected = false;
    switch(this.props.connection.status) {
    case 'connecting': isConnecting = true; break;
    case 'connected': isConnected = true; break;
    case 'disconnected': isDisconnected = true; break;
    }

    const { city, country } = serverInfo && (isConnecting || isConnected)
      ? serverInfo
      : { city: '\u2003', country: '\u2002' };
    const ip = serverInfo && isConnected
      ? serverInfo.address
      : '\u2003'; //this.props.connection.clientIp;
    const serverName = serverInfo
      ? serverInfo.name
      : '\u2003';

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

    let ipComponent = undefined;
    if (isConnected || isDisconnected) {
      if (this.state.showCopyIPMessage) {
        ipComponent = <span>{ 'IP copied to clipboard!' }</span>;
      } else {
        ipComponent = <span>{ ip }</span>;
      }
    }
    return (
      <div className="connect">
        <div className="connect__map">
          { map }
        </div>
        <div className="connect__container">

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

            { /* location when connecting */ }
            <If condition={ isConnecting }>
              <Then>
                <div className="connect__status-location">
                  <span>{ country }</span>
                </div>
              </Then>
            </If>

            { /* location when connected */ }
            <If condition={ isConnected }>
              <Then>
                <div className="connect__status-location">
                  { city }<br/>{ country }
                </div>
              </Then>
            </If>

            { /* location when disconnected */ }
            <If condition={ isDisconnected }>
              <Then>
                <div className="connect__status-location">
                  { country }
                </div>
              </Then>
            </If>

            { /*
              **********************************
              End: Location block
              **********************************
            */ }

            <div className={ this.ipAddressClass() } onClick={ this.onIPAddressClick.bind(this) }>
              { ipComponent }
            </div>
          </div>


          { /*
            **********************************
            Begin: Footer block
            **********************************
          */ }

          { /* footer when disconnected */ }
          <If condition={ isDisconnected }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">

                  <div className="connect__server" onClick={ this.props.onSelectLocation }>
                    <div className="connect__server-label">Connect to</div>
                    <div className="connect__server-value">

                      <div className="connect__server-name">{ serverName }</div>

                    </div>
                  </div>
                </div>

                <div className="connect__row">
                  <button className="button button--positive" onClick={ this.onConnect.bind(this) }>Secure my connection</button>
                </div>
              </div>
            </Then>
          </If>

          { /* footer when connecting */ }
          <If condition={ isConnecting }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">
                  <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
                </div>

                <div className="connect__row">
                  <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Cancel</button>
                </div>
              </div>
            </Then>
          </If>

          { /* footer when connected */ }
          <If condition={ isConnected }>
            <Then>
              <div className="connect__footer">
                <div className="connect__row">
                  <button className="button button--neutral button--blur" onClick={ this.props.onSelectLocation }>Switch location</button>
                </div>

                <div className="connect__row">
                  <button className="button button--negative-light button--blur" onClick={ this.props.onDisconnect }>Disconnect</button>
                </div>
              </div>
            </Then>
          </If>

          { /*
            **********************************
            End: Footer block
            **********************************
          */ }

        </div>
      </div>
    );
  }

  // Handlers

  onConnect() {
    const serverInfo = this._getServerInfo();
    if(!serverInfo) {
      return;
    }

    this.props.onConnect(serverInfo.address);
  }

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
    case 'connecting':
    case 'disconnected':
      return 'error';
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
