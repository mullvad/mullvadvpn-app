// @flow

import log from 'electron-log';
import { remote, webFrame, ipcRenderer } from 'electron';
import * as React from 'react';
import { bindActionCreators } from 'redux';
import { Provider } from 'react-redux';
import {
  ConnectedRouter,
  push as pushHistory,
  replace as replaceHistory,
} from 'connected-react-router';
import { createMemoryHistory } from 'history';

import makeRoutes from './routes';
import ReconnectionBackoff from './lib/reconnection-backoff';
import { DaemonRpc } from './lib/daemon-rpc';
import NotificationController from './lib/notification-controller';
import setShutdownHandler from './lib/shutdown-handler';
import { NoAccountError } from './errors';

import configureStore from './redux/store';
import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import versionActions from './redux/version/actions';
import windowActions from './redux/window/actions';

import type { WindowShapeParameters } from '../main/window-controller';
import type {
  AccountToken,
  TunnelStateTransition,
  RelaySettingsUpdate,
  TunnelState,
  DaemonRpcProtocol,
  AccountData,
  ConnectionObserver as DaemonConnectionObserver,
} from './lib/daemon-rpc';
import type { ReduxStore } from './redux/store';
import type { TrayIconType } from '../main/tray-icon-controller';

export default class AppRenderer {
  _notificationController = new NotificationController();
  _daemonRpc: DaemonRpcProtocol = new DaemonRpc();
  _reconnectBackoff = new ReconnectionBackoff();
  _openConnectionObserver: ?DaemonConnectionObserver;
  _closeConnectionObserver: ?DaemonConnectionObserver;
  _memoryHistory = createMemoryHistory();
  _reduxStore: ReduxStore;
  _reduxActions: *;
  _accountDataCache = new AccountDataCache((accountToken) => {
    return this._daemonRpc.getAccountData(accountToken);
  });
  _connectedToDaemon = false;
  _accountToken: ?AccountToken;
  _tunnelState: ?TunnelStateTransition;

  constructor() {
    const store = configureStore(null, this._memoryHistory);
    const dispatch = store.dispatch;

    this._reduxStore = store;
    this._reduxActions = {
      account: bindActionCreators(accountActions, dispatch),
      connection: bindActionCreators(connectionActions, dispatch),
      settings: bindActionCreators(settingsActions, dispatch),
      version: bindActionCreators(versionActions, dispatch),
      window: bindActionCreators(windowActions, dispatch),
      history: bindActionCreators(
        {
          push: pushHistory,
          replace: replaceHistory,
        },
        dispatch,
      ),
    };

    this._openConnectionObserver = this._daemonRpc.addOpenConnectionObserver(() => {
      this._onOpenConnection();
    });

    this._closeConnectionObserver = this._daemonRpc.addCloseConnectionObserver((error) => {
      this._onCloseConnection(error);
    });

    setShutdownHandler(async () => {
      log.info('Executing a shutdown handler');
      try {
        await this.disconnectTunnel();
        log.info('Disconnected the tunnel');
      } catch (e) {
        log.error(`Failed to shutdown tunnel: ${e.message}`);
      }
    });

    ipcRenderer.on('update-window-shape', (_event, shapeParams: WindowShapeParameters) => {
      if (typeof shapeParams.arrowPosition === 'number') {
        this._reduxActions.window.updateWindowArrowPosition(shapeParams.arrowPosition);
      }
    });

    // disable pinch to zoom
    webFrame.setVisualZoomLevelLimits(1, 1);
  }

  dispose() {
    const openConnectionObserver = this._openConnectionObserver;
    const closeConnectionObserver = this._closeConnectionObserver;

    if (openConnectionObserver) {
      openConnectionObserver.unsubscribe();
      this._openConnectionObserver = null;
    }

    if (closeConnectionObserver) {
      closeConnectionObserver.unsubscribe();
      this._closeConnectionObserver = null;
    }
  }

  renderView() {
    return (
      <Provider store={this._reduxStore}>
        <ConnectedRouter history={this._memoryHistory}>{makeRoutes({ app: this })}</ConnectedRouter>
      </Provider>
    );
  }

  connect() {
    this._connectToDaemon();
  }

  disconnect() {
    this._daemonRpc.disconnect();
  }

  async login(accountToken: AccountToken) {
    const actions = this._reduxActions;
    const history = this._memoryHistory;
    actions.account.startLogin(accountToken);

    log.debug('Logging in');

    try {
      const accountData = await this._daemonRpc.getAccountData(accountToken);
      await this._daemonRpc.setAccount(accountToken);

      // reset the account token with the one that we just sent to daemon
      this._accountToken = accountToken;

      actions.account.updateAccountExpiry(accountData.expiry);
      actions.account.loginSuccessful();

      // Redirect the user after some time to allow for
      // the 'Login Successful' screen to be visible
      setTimeout(async () => {
        if (history.location.pathname === '/login') {
          actions.history.replace('/connect');
        }

        try {
          log.debug('Auto-connecting the tunnel');
          await this.connectTunnel();
        } catch (error) {
          log.error(`Failed to auto-connect the tunnel: ${error.message}`);
        }
      }, 1000);
    } catch (error) {
      log.error('Failed to log in,', error.message);

      actions.account.loginFailed(error);
    }
  }

  async _restoreSession() {
    const actions = this._reduxActions;
    const history = this._memoryHistory;

    log.debug('Restoring session');

    const accountToken = await this._daemonRpc.getAccount();

    // save the account token received after connecting to daemon
    this._accountToken = accountToken;

    if (accountToken) {
      log.debug(`Got account token: ${accountToken}`);
      actions.account.updateAccountToken(accountToken);
      actions.account.loginSuccessful();

      // take user to main view if user is still at launch screen `/`
      if (history.location.pathname === '/') {
        actions.history.replace('/connect');
      } else {
        // TODO: Reinvent the navigation back in history to make sure that user does not end up on
        // the restricted screen due to changes in daemon's state.
        for (const entry of history.entries) {
          if (entry.pathname === '/') {
            entry.pathname = '/connect';
          }
        }
      }
    } else {
      log.debug('No account set, showing login view.');

      // show window when account is not set
      ipcRenderer.send('show-window');

      // take user to `/login` screen if user is at launch screen `/`
      if (history.location.pathname === '/') {
        actions.history.replace('/login');
      } else {
        // TODO: Reinvent the navigation back in history to make sure that user does not end up on
        // the restricted screen due to changes in daemon's state.
        for (const entry of history.entries) {
          if (!entry.pathname.startsWith('/settings')) {
            entry.pathname = '/login';
          }
        }
      }
    }
  }

  async logout() {
    const actions = this._reduxActions;

    try {
      await this._daemonRpc.setAccount(null);

      // reset the account token after log out
      this._accountToken = null;

      await this._fetchAccountHistory();

      actions.account.loggedOut();
      actions.history.replace('/login');

      // invalidate account data cache on log out
      this._accountDataCache.invalidate();
    } catch (e) {
      log.info('Failed to logout: ', e.message);
    }
  }

  async connectTunnel() {
    const actions = this._reduxActions;

    // connect only if tunnel is disconnected or blocked.
    if (
      this._tunnelState &&
      (this._tunnelState.state === 'disconnected' || this._tunnelState.state === 'blocked')
    ) {
      // switch to connecting state immediately to prevent a lag that may be caused by RPC
      // communication delay
      actions.connection.connecting();

      await this._daemonRpc.connectTunnel();
    }
  }

  disconnectTunnel() {
    return this._daemonRpc.disconnectTunnel();
  }

  updateRelaySettings(relaySettings: RelaySettingsUpdate) {
    return this._daemonRpc.updateRelaySettings(relaySettings);
  }

  async fetchRelaySettings() {
    const actions = this._reduxActions;
    const relaySettings = await this._daemonRpc.getRelaySettings();

    log.debug('Got relay settings from daemon', JSON.stringify(relaySettings));

    if (relaySettings.normal) {
      const payload = {};
      const normal = relaySettings.normal;
      const tunnel = normal.tunnel;
      const location = normal.location;

      if (location === 'any') {
        payload.location = 'any';
      } else {
        payload.location = location.only;
      }

      if (tunnel === 'any') {
        payload.port = 'any';
        payload.protocol = 'any';
      } else {
        const { port, protocol } = tunnel.only.openvpn;
        payload.port = port === 'any' ? port : port.only;
        payload.protocol = protocol === 'any' ? protocol : protocol.only;
      }

      actions.settings.updateRelay({
        normal: payload,
      });
    } else if (relaySettings.custom_tunnel_endpoint) {
      const custom_tunnel_endpoint = relaySettings.custom_tunnel_endpoint;
      const {
        host,
        tunnel: {
          openvpn: { port, protocol },
        },
      } = custom_tunnel_endpoint;

      actions.settings.updateRelay({
        custom_tunnel_endpoint: {
          host,
          port,
          protocol,
        },
      });
    }
  }

  async updateAccountExpiry() {
    const actions = this._reduxActions;
    const accountDataCache = this._accountDataCache;

    try {
      const accountToken = this._accountToken;
      if (accountToken) {
        const accountData = await accountDataCache.fetch(accountToken);
        actions.account.updateAccountExpiry(accountData.expiry);
      } else {
        throw new NoAccountError();
      }
    } catch (e) {
      log.error(`Failed to update account expiry: ${e.message}`);
    }
  }

  async removeAccountFromHistory(accountToken: AccountToken): Promise<void> {
    await this._daemonRpc.removeAccountFromHistory(accountToken);
    await this._fetchAccountHistory();
  }

  async _fetchAccountHistory(): Promise<void> {
    const actions = this._reduxActions;

    const accountHistory = await this._daemonRpc.getAccountHistory();
    actions.account.updateAccountHistory(accountHistory);
  }

  async _fetchRelayLocations() {
    const actions = this._reduxActions;
    const locations = await this._daemonRpc.getRelayLocations();

    log.info('Got relay locations');

    const storedLocations = locations.countries.map((country) => ({
      name: country.name,
      code: country.code,
      hasActiveRelays: country.cities.some((city) => city.relays.length > 0),
      cities: country.cities.map((city) => ({
        name: city.name,
        code: city.code,
        latitude: city.latitude,
        longitude: city.longitude,
        hasActiveRelays: city.relays.length > 0,
        relays: city.relays,
      })),
    }));

    actions.settings.updateRelayLocations(storedLocations);
  }

  async _fetchLocation() {
    const actions = this._reduxActions;
    const location = await this._daemonRpc.getLocation();

    log.info('Got location from daemon');

    const locationUpdate = {
      ip: location.ip,
      country: location.country,
      city: location.city,
      latitude: location.latitude,
      longitude: location.longitude,
      mullvadExitIp: location.mullvad_exit_ip,
    };

    actions.connection.newLocation(locationUpdate);
  }

  async setAllowLan(allowLan: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setAllowLan(allowLan);
    actions.settings.updateAllowLan(allowLan);
  }

  async setEnableIpv6(enableIpv6: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setEnableIpv6(enableIpv6);
    actions.settings.updateEnableIpv6(enableIpv6);
  }

  async setAutoConnect(autoConnect: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setAutoConnect(autoConnect);
    actions.settings.updateAutoConnect(autoConnect);
  }

  async _fetchAllowLan() {
    const actions = this._reduxActions;
    const allowLan = await this._daemonRpc.getAllowLan();
    actions.settings.updateAllowLan(allowLan);
  }

  async _fetchAutoConnect() {
    const actions = this._reduxActions;
    const autoConnect = await this._daemonRpc.getAutoConnect();
    actions.settings.updateAutoConnect(autoConnect);
  }

  async _fetchTunnelState() {
    const state = await this._daemonRpc.getState();

    log.debug(`Got state: ${JSON.stringify(state)}`);

    this._onChangeTunnelState(state);
  }

  async _fetchTunnelOptions() {
    const actions = this._reduxActions;
    const tunnelOptions = await this._daemonRpc.getTunnelOptions();
    actions.settings.updateEnableIpv6(tunnelOptions.enableIpv6);
  }

  async _fetchVersionInfo() {
    const actions = this._reduxActions;
    const latestVersionInfo = await this._daemonRpc.getVersionInfo();
    const versionFromDaemon = await this._daemonRpc.getCurrentVersion();
    const versionFromGui = remote.app.getVersion().replace('.0', '');

    actions.version.updateVersion(versionFromDaemon, versionFromDaemon === versionFromGui);
    actions.version.updateLatest(latestVersionInfo);
  }

  async _connectToDaemon(): Promise<void> {
    this._daemonRpc.connect({ path: getIpcPath() });
  }

  async _onOpenConnection() {
    this._connectedToDaemon = true;

    // reset the reconnect backoff when connection established.
    this._reconnectBackoff.reset();

    // attempt to restore the session
    try {
      await this._restoreSession();
    } catch (error) {
      log.error(`Failed to restore session: ${error.message}`);
    }

    // make sure to re-subscribe to state notifications when connection is re-established.
    try {
      await this._subscribeStateListener();
    } catch (error) {
      log.error(`Cannot subscribe for RPC notifications: ${error.message}`);
    }

    // fetch initial state
    try {
      await this._fetchInitialState();
    } catch (error) {
      log.error(`Cannot fetch initial state: ${error.message}`);
    }

    // auto connect the tunnel on startup
    // note: disabled when developing
    if (process.env.NODE_ENV !== 'development') {
      // only connect if account is set in the daemon
      if (this._accountToken) {
        try {
          log.debug('Autoconnect the tunnel');
          await this.connectTunnel();
        } catch (error) {
          log.error(`Failed to autoconnect the tunnel: ${error.message}`);
        }
      } else {
        log.debug('Skip autoconnect because account token is not set');
      }
    } else {
      log.debug('Skip autoconnect in development');
    }
  }

  async _onCloseConnection(error: ?Error) {
    const actions = this._reduxActions;

    // recover connection on error
    if (error) {
      log.debug(`Lost connection to daemon: ${error.message}`);

      const recover = async () => {
        try {
          await this.connect();
        } catch (error) {
          log.error(`Failed to reconnect: ${error.message}`);
        }
      };

      this._reconnectBackoff.attempt(() => {
        recover();
      });

      // only send to the connecting to daemon view if the daemon was
      // connnected previously
      if (this._connectedToDaemon) {
        actions.history.replace('/');
      }
    } else {
      log.info(`Disconnected from the daemon`);
    }
    this._connectedToDaemon = false;
  }

  async _subscribeStateListener() {
    await this._daemonRpc.subscribeStateListener((newState, error) => {
      if (error) {
        log.error(`Failed to deserialize the new state: ${error.message}`);
      }

      if (newState) {
        log.debug(`Got state update: '${JSON.stringify(newState)}'`);

        this._onChangeTunnelState(newState);
      }
    });
  }

  _fetchInitialState() {
    return Promise.all([
      this._fetchTunnelState(),
      this.fetchRelaySettings(),
      this._fetchRelayLocations(),
      this._fetchAllowLan(),
      this._fetchAutoConnect(),
      this._fetchLocation(),
      this._fetchAccountHistory(),
      this._fetchTunnelOptions(),
      this._fetchVersionInfo(),
    ]);
  }

  _onChangeTunnelState(tunnelState: TunnelStateTransition) {
    this._tunnelState = tunnelState;

    this._updateConnectionStatus(tunnelState);
    this._updateUserLocation(tunnelState.state);
    this._updateTrayIcon(tunnelState.state);
    this._showNotification(tunnelState.state);
  }

  async _updateUserLocation(tunnelState: TunnelState) {
    if (tunnelState === 'connecting' || tunnelState === 'disconnected') {
      try {
        await this._fetchLocation();
      } catch (error) {
        log.error(`Failed to update the location: ${error.message}`);
      }
    }
  }

  _updateConnectionStatus(stateTransition: TunnelStateTransition) {
    const actions = this._reduxActions;

    switch (stateTransition.state) {
      case 'connecting':
        actions.connection.connecting();
        break;

      case 'connected':
        actions.connection.connected();
        break;

      case 'disconnecting':
        actions.connection.disconnecting();
        break;

      case 'disconnected':
        actions.connection.disconnected();
        break;

      case 'blocked':
        actions.connection.blocked(stateTransition.details);
        break;

      default:
        log.error(`Unexpected TunnelStateTransition: ${(stateTransition.state: empty)}`);
        break;
    }
  }

  _updateTrayIcon(tunnelState: TunnelState) {
    const iconTypes: { [TunnelState]: TrayIconType } = {
      connected: 'secured',
      connecting: 'securing',
      blocked: 'securing',
    };
    const type = iconTypes[tunnelState] || 'unsecured';

    ipcRenderer.send('change-tray-icon', type);
  }

  _showNotification(tunnelState: TunnelState) {
    switch (tunnelState) {
      case 'connecting':
        this._notificationController.show('Connecting');
        break;
      case 'connected':
        this._notificationController.show('Secured');
        break;
      case 'disconnected':
        this._notificationController.show('Unsecured');
        break;
      case 'blocked':
        this._notificationController.show('Blocked all connections');
        break;
      case 'disconnecting':
        // no-op
        break;
      default:
        log.error(`Unexpected TunnelState: ${(tunnelState: empty)}`);
        return;
    }
  }
}

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
class AccountDataCache {
  _executingPromise: ?Promise<AccountData>;
  _value: ?AccountData;
  _expiresAt: ?Date;
  _fetch: (AccountToken) => Promise<AccountData>;

  constructor(fetch: (AccountToken) => Promise<AccountData>) {
    this._fetch = fetch;
  }

  async fetch(accountToken: AccountToken): Promise<AccountData> {
    // return the same promise if still fetching from remote
    const executingPromise = this._executingPromise;
    if (executingPromise) {
      return executingPromise;
    }

    // return the received value if not expired yet
    const currentValue = this._value;
    if (currentValue && !this._isExpired()) {
      return currentValue;
    }

    try {
      const nextPromise = this._fetch(accountToken);
      this._executingPromise = nextPromise;

      const accountData = await nextPromise;

      // it's possible that invalidate() was called before this promise was resolved.
      // discard the result of "orphaned" promise.
      if (this._executingPromise === nextPromise) {
        this._setValue(accountData);
        return accountData;
      } else {
        throw new Error('Cancelled');
      }
    } catch (error) {
      throw error;
    } finally {
      this._executingPromise = null;
    }
  }

  invalidate() {
    this._executingPromise = null;
    this._expiresAt = null;
    this._value = null;
  }

  _setValue(value: AccountData) {
    this._expiresAt = new Date(Date.now() + 60 * 1000); // 60s expiration
    this._value = value;
  }

  _isExpired() {
    return !this._expiresAt || this._expiresAt < new Date();
  }
}

const getIpcPath = (): string => {
  if (process.platform === 'win32') {
    return '//./pipe/Mullvad VPN';
  } else {
    return '/var/run/mullvad-vpn';
  }
};
