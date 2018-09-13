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
import { DaemonRpc, ConnectionObserver } from './lib/daemon-rpc';
import NotificationController from './lib/notification-controller';
import setShutdownHandler from './lib/shutdown-handler';

import configureStore from './redux/store';
import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import versionActions from './redux/version/actions';
import windowActions from './redux/window/actions';

import SettingsProxy from './lib/subscription-proxy/settings-proxy';
import TunnelStateProxy from './lib/subscription-proxy/tunnel-state-proxy';

import type { WindowShapeParameters } from '../main/window-controller';
import type {
  AccountToken,
  Settings,
  TunnelStateTransition,
  RelaySettingsUpdate,
  RelaySettings,
  TunnelState,
  DaemonRpcProtocol,
  AccountData,
} from './lib/daemon-rpc';
import type { ReduxStore } from './redux/store';
import type { TrayIconType } from '../main/tray-icon-controller';

export default class AppRenderer {
  _notificationController = new NotificationController();
  _daemonRpc: DaemonRpcProtocol = new DaemonRpc();
  _connectionObserver = new ConnectionObserver(
    () => {
      this._onOpenConnection();
    },
    (error) => {
      this._onCloseConnection(error);
    },
  );
  _reconnectBackoff = new ReconnectionBackoff();
  _memoryHistory = createMemoryHistory();
  _reduxStore: ReduxStore;
  _reduxActions: *;
  _accountDataCache = new AccountDataCache((accountToken) => {
    return this._daemonRpc.getAccountData(accountToken);
  });
  _tunnelStateProxy = new TunnelStateProxy(this._daemonRpc, (tunnelState) => {
    this._setTunnelState(tunnelState);
  });
  _settingsProxy = new SettingsProxy(this._daemonRpc, (settings) => {
    this._setSettings(settings);
  });
  _connectedToDaemon = false;

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

    this._daemonRpc.addConnectionObserver(this._connectionObserver);

    setShutdownHandler(async () => {
      log.info('Executing a shutdown handler');
      try {
        await this.disconnectTunnel();
        log.info('Disconnected the tunnel');
      } catch (e) {
        log.error(`Failed to disconnect the tunnel: ${e.message}`);
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

  renderView() {
    return (
      <Provider store={this._reduxStore}>
        <ConnectedRouter history={this._memoryHistory}>{makeRoutes({ app: this })}</ConnectedRouter>
      </Provider>
    );
  }

  connect() {
    this._daemonRpc.connect({ path: getIpcPath() });
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

  async logout() {
    const actions = this._reduxActions;

    try {
      await this._daemonRpc.setAccount(null);

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

    const tunnelState = await this._tunnelStateProxy.fetch();

    // connect only if tunnel is disconnected or blocked.
    if (tunnelState.state === 'disconnected' || tunnelState.state === 'blocked') {
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

  _setRelaySettings(relaySettings: RelaySettings) {
    const actions = this._reduxActions;

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
    } else if (relaySettings.customTunnelEndpoint) {
      const customTunnelEndpoint = relaySettings.customTunnelEndpoint;
      const {
        host,
        tunnel: {
          openvpn: { port, protocol },
        },
      } = customTunnelEndpoint;

      actions.settings.updateRelay({
        customTunnelEndpoint: {
          host,
          port,
          protocol,
        },
      });
    }
  }

  async updateAccountExpiry() {
    const actions = this._reduxActions;
    const settings = await this._settingsProxy.fetch();
    const accountDataCache = this._accountDataCache;

    if (settings && settings.accountToken) {
      try {
        const accountData = await accountDataCache.fetch(settings.accountToken);
        actions.account.updateAccountExpiry(accountData.expiry);
      } catch (error) {
        log.error(`Failed to update account expiry: ${error.message}`);
      }
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

  async _fetchVersionInfo() {
    const actions = this._reduxActions;
    const latestVersionInfo = await this._daemonRpc.getVersionInfo();
    const versionFromDaemon = await this._daemonRpc.getCurrentVersion();
    const versionFromGui = remote.app.getVersion().replace('.0', '');

    actions.version.updateVersion(versionFromDaemon, versionFromDaemon === versionFromGui);
    actions.version.updateLatest(latestVersionInfo);
  }

  async _onOpenConnection() {
    this._connectedToDaemon = true;

    // reset the reconnect backoff when connection established.
    this._reconnectBackoff.reset();

    try {
      await this._runPrimaryApplicationFlow();
    } catch (error) {
      log.error(`An error was raised when running the primary application flow: ${error.message}`);
    }
  }

  async _runPrimaryApplicationFlow() {
    // fetch initial state and subscribe for changes
    try {
      await this._settingsProxy.fetch();
    } catch (error) {
      log.error(`Cannot fetch the initial settings: ${error.message}`);
    }

    try {
      await this._tunnelStateProxy.fetch();
    } catch (error) {
      log.error(`Cannot fetch the initial tunnel state: ${error.message}`);
    }

    // fetch the rest of data
    try {
      await this._fetchRelayLocations();
    } catch (error) {
      log.error(`Cannot fetch the relay locations: ${error.message}`);
    }

    try {
      await this._fetchAccountHistory();
    } catch (error) {
      log.error(`Cannot fetch the account history: ${error.message}`);
    }

    // set the appropriate start view
    await this._setStartView();

    try {
      await this._fetchLocation();
    } catch (error) {
      log.error(`Cannot fetch the location: ${error.message}`);
    }

    try {
      await this._fetchVersionInfo();
    } catch (error) {
      log.error(`Cannot fetch the version information: ${error.message}`);
    }

    // auto connect the tunnel on startup
    // note: disabled when developing
    if (process.env.NODE_ENV !== 'development') {
      const settings = await this._settingsProxy.fetch();

      // only connect if account is set in the daemon
      if (settings.accountToken) {
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

  async _setStartView() {
    const actions = this._reduxActions;
    const history = this._memoryHistory;
    const settings = await this._settingsProxy.fetch();
    const accountToken = settings.accountToken;

    if (accountToken) {
      log.debug(`Account token is set. Showing the tunnel view.`);

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

  _onCloseConnection(error: ?Error) {
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

  _setTunnelState(tunnelState: TunnelStateTransition) {
    log.debug(`Tunnel state: ${tunnelState.state}`);

    this._updateConnectionStatus(tunnelState);
    this._updateUserLocation(tunnelState.state);
    this._updateTrayIcon(tunnelState.state);
    this._showNotification(tunnelState.state);
  }

  _setSettings(newSettings: Settings) {
    const reduxSettings = this._reduxActions.settings;

    reduxSettings.updateAllowLan(newSettings.allowLan);
    reduxSettings.updateAutoConnect(newSettings.autoConnect);
    reduxSettings.updateEnableIpv6(newSettings.tunnelOptions.enableIpv6);

    this._setRelaySettings(newSettings.relaySettings);
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

function getIpcPath(): string {
  if (process.platform === 'win32') {
    return '//./pipe/Mullvad VPN';
  } else {
    return '/var/run/mullvad-vpn';
  }
}
