// @flow

import log from 'electron-log';
import { webFrame, ipcRenderer } from 'electron';
import * as React from 'react';
import { bindActionCreators } from 'redux';
import { Provider } from 'react-redux';
import {
  ConnectedRouter,
  push as pushHistory,
  replace as replaceHistory,
} from 'connected-react-router';
import { createMemoryHistory } from 'history';

import { InvalidAccountError } from '../main/errors';
import makeRoutes from './routes';

import configureStore from './redux/store';
import accountActions from './redux/account/actions';
import connectionActions from './redux/connection/actions';
import settingsActions from './redux/settings/actions';
import versionActions from './redux/version/actions';
import userInterfaceActions from './redux/userinterface/actions';

import type { WindowShapeParameters } from '../main/window-controller';
import type { CurrentAppVersionInfo, AppUpgradeInfo } from '../main';
import type { GuiSettingsState } from '../shared/gui-settings-state';
import IpcEventChannel from '../shared/ipc-event-channel';

import type {
  AccountToken,
  AccountData,
  Location,
  RelayList,
  RelaySettingsUpdate,
  RelaySettings,
  Settings,
  TunnelStateTransition,
} from './lib/daemon-rpc-proxy';

import DaemonRpcProxy from './lib/daemon-rpc-proxy';

import type { ReduxStore } from './redux/store';

export default class AppRenderer {
  _memoryHistory = createMemoryHistory();
  _reduxStore: ReduxStore;
  _reduxActions: *;
  _daemonRpc = new DaemonRpcProxy();
  _accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this._daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      const expiry = accountData ? accountData.expiry : null;
      this._reduxActions.account.updateAccountExpiry(expiry);
    },
  );

  _tunnelState: TunnelStateTransition;
  _settings: Settings;
  _connectedToDaemon = false;
  _uncoupledFromTunnel = false;

  constructor() {
    const store = configureStore(null, this._memoryHistory);
    const dispatch = store.dispatch;

    this._reduxStore = store;
    this._reduxActions = {
      account: bindActionCreators(accountActions, dispatch),
      connection: bindActionCreators(connectionActions, dispatch),
      settings: bindActionCreators(settingsActions, dispatch),
      version: bindActionCreators(versionActions, dispatch),
      userInterface: bindActionCreators(userInterfaceActions, dispatch),
      history: bindActionCreators(
        {
          push: pushHistory,
          replace: replaceHistory,
        },
        dispatch,
      ),
    };

    ipcRenderer.on('update-window-shape', (_event, shapeParams: WindowShapeParameters) => {
      if (typeof shapeParams.arrowPosition === 'number') {
        this._reduxActions.userInterface.updateWindowArrowPosition(shapeParams.arrowPosition);
      }
    });

    ipcRenderer.on('window-shown', () => {
      if (this._connectedToDaemon) {
        this.updateAccountExpiry();
      }
    });

    IpcEventChannel.daemonConnected.listen(() => {
      this._onDaemonConnected();
    });

    IpcEventChannel.daemonDisconnected.listen((errorMessage: ?string) => {
      this._onDaemonDisconnected(errorMessage ? new Error(errorMessage) : null);
    });

    IpcEventChannel.tunnelState.listen((newState: TunnelStateTransition) => {
      this._setTunnelState(newState);
    });

    IpcEventChannel.settings.listen((newSettings: Settings) => {
      this._setSettings(newSettings);
    });

    IpcEventChannel.location.listen((newLocation: Location) => {
      this._setLocation(newLocation);
    });

    IpcEventChannel.relays.listen((newRelays: RelayList) => {
      this._setRelays(newRelays);
    });

    IpcEventChannel.currentVersion.listen((currentVersion: CurrentAppVersionInfo) => {
      this._setCurrentVersion(currentVersion);
    });

    IpcEventChannel.upgradeVersion.listen((upgradeVersion: AppUpgradeInfo) => {
      this._setUpgradeVersion(upgradeVersion);
    });

    IpcEventChannel.guiSettings.listen((guiSettings: GuiSettingsState) => {
      this._setGuiSettings(guiSettings);
    });

    // Request the initial state from the main process
    const initialState = IpcEventChannel.state.get();

    this._setTunnelState(initialState.tunnelState);
    this._setSettings(initialState.settings);

    if (initialState.location) {
      this._setLocation(initialState.location);
    }

    this._setRelays(initialState.relays);
    this._setCurrentVersion(initialState.currentVersion);
    this._setUpgradeVersion(initialState.upgradeVersion);
    this._setGuiSettings(initialState.guiSettings);

    if (initialState.isConnected) {
      this._onDaemonConnected();
    }

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

  async login(accountToken: AccountToken) {
    const actions = this._reduxActions;
    const history = this._memoryHistory;
    actions.account.startLogin(accountToken);

    log.debug('Logging in');

    try {
      const verification = await this.verifyAccount(accountToken);

      if (verification.status === 'deferred') {
        log.debug(`Failed to get account data, logging in anyway: ${verification.error.message}`);
      }

      await this._daemonRpc.setAccount(accountToken);
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

  async verifyAccount(accountToken: AccountToken): Promise<AccountVerification> {
    return new Promise((resolve, reject) => {
      this._accountDataCache.invalidate();
      this._accountDataCache.fetch(accountToken, {
        onFinish: () => resolve({ status: 'verified' }),
        onError: (error): AccountFetchRetryAction => {
          if (error instanceof InvalidAccountError) {
            reject(error);
            return 'stop';
          } else {
            resolve({ status: 'deferred', error });
            return 'retry';
          }
        },
      });
    });
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

  async connectTunnel(): Promise<void> {
    const state = this._tunnelState.state;

    // connect only if tunnel is disconnected or blocked.
    if (state === 'disconnecting' || state === 'disconnected' || state === 'blocked') {
      // switch to the connecting state ahead of time to make the app look more responsive
      this._reduxActions.connection.connecting(null);

      return this._daemonRpc.connectTunnel();
    }
  }

  disconnectTunnel(): Promise<void> {
    // switch to the disconnected state ahead of time to make the app look more responsive
    this._reduxActions.connection.disconnected();

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
    if (this._settings.accountToken) {
      this._accountDataCache.fetch(this._settings.accountToken);
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

  async setBlockWhenDisconnected(blockWhenDisconnected: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setBlockWhenDisconnected(blockWhenDisconnected);
    actions.settings.updateBlockWhenDisconnected(blockWhenDisconnected);
  }

  async setOpenVpnMssfix(mssfix: ?number) {
    const actions = this._reduxActions;
    actions.settings.updateOpenVpnMssfix(mssfix);
    await this._daemonRpc.setOpenVpnMssfix(mssfix);
  }

  async setAutoConnect(autoConnect: boolean) {
    const actions = this._reduxActions;
    await this._daemonRpc.setAutoConnect(autoConnect);
    actions.settings.updateAutoConnect(autoConnect);
  }

  async setStartMinimized(startMinimized: boolean) {
    IpcEventChannel.guiSettings.setStartMinimized(startMinimized);
  }

  async setUncoupledFromTunnel(uncoupledFromTunnel: boolean) {
    IpcEventChannel.guiSettings.setUncoupledFromTunnel(uncoupledFromTunnel);
  }

  async _onDaemonConnected() {
    this._connectedToDaemon = true;

    try {
      await this._fetchAccountHistory();
    } catch (error) {
      log.error(`Cannot fetch the account history: ${error.message}`);
    }

    // set the appropriate start view
    await this._setStartView();
    // try to autoconnect the tunnel
    await this._autoConnect();
  }

  _onDaemonDisconnected(error: ?Error) {
    const actions = this._reduxActions;

    // recover connection on error
    if (error) {
      // only send to the connecting to daemon view if the daemon was
      // connnected previously
      if (this._connectedToDaemon) {
        actions.history.replace('/');
      }
    }

    this._connectedToDaemon = false;
  }

  async _setStartView() {
    const actions = this._reduxActions;
    const history = this._memoryHistory;
    const accountToken = this._settings.accountToken;

    if (accountToken) {
      log.debug(`Account token is set. Showing the tunnel view.`);

      this._accountDataCache.fetch(accountToken);

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

  async _autoConnect() {
    const accountToken = this._settings.accountToken;

    if (accountToken) {
      if (process.env.NODE_ENV === 'development') {
        log.debug('Skip autoconnect in development');
      } else if (this._uncoupledFromTunnel) {
        log.debug('Skip autoconnect because app is uncoupled from tunnel');
      } else {
        try {
          log.debug('Autoconnect the tunnel');
          await this.connectTunnel();
        } catch (error) {
          log.error(`Failed to autoconnect the tunnel: ${error.message}`);
        }
      }
    } else {
      log.debug('Skip autoconnect because account token is not set');
    }
  }

  _setTunnelState(tunnelState: TunnelStateTransition) {
    const actions = this._reduxActions;

    log.debug(`Tunnel state: ${tunnelState.state}`);

    this._tunnelState = tunnelState;

    switch (tunnelState.state) {
      case 'connecting':
        actions.connection.connecting(tunnelState.details);
        break;

      case 'connected':
        actions.connection.connected(tunnelState.details);
        break;

      case 'disconnecting':
        actions.connection.disconnecting(tunnelState.details);
        break;

      case 'disconnected':
        actions.connection.disconnected();
        break;

      case 'blocked':
        actions.connection.blocked(tunnelState.details);
        break;

      default:
        log.error(`Unexpected TunnelState: ${(tunnelState.state: empty)}`);
        break;
    }
  }

  _setSettings(newSettings: Settings) {
    this._settings = newSettings;

    const reduxSettings = this._reduxActions.settings;

    reduxSettings.updateAllowLan(newSettings.allowLan);
    reduxSettings.updateAutoConnect(newSettings.autoConnect);
    reduxSettings.updateEnableIpv6(newSettings.tunnelOptions.enableIpv6);
    reduxSettings.updateBlockWhenDisconnected(newSettings.blockWhenDisconnected);
    reduxSettings.updateOpenVpnMssfix(newSettings.tunnelOptions.openvpn.mssfix);

    this._setRelaySettings(newSettings.relaySettings);
  }

  _setLocation(location: Location) {
    this._reduxActions.connection.newLocation(location);
  }

  _setRelays(relayList: RelayList) {
    const locations = relayList.countries.map((country) => ({
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

    this._reduxActions.settings.updateRelayLocations(locations);
  }

  _setCurrentVersion(versionInfo: CurrentAppVersionInfo) {
    this._reduxActions.version.updateVersion(versionInfo.gui, versionInfo.isConsistent);
  }

  _setUpgradeVersion(upgradeVersion: AppUpgradeInfo) {
    this._reduxActions.version.updateLatest(upgradeVersion);
  }

  _setGuiSettings(guiSettings: GuiSettingsState) {
    this._uncoupledFromTunnel = guiSettings.uncoupledFromTunnel;
    this._reduxActions.settings.updateGuiSettings(guiSettings);
  }
}

type AccountVerification = { status: 'verified' } | { status: 'deferred', error: Error };
type AccountFetchRetryAction = 'stop' | 'retry';
type AccountFetchWatcher = {
  onFinish: () => void,
  onError: (any) => AccountFetchRetryAction,
};

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
export class AccountDataCache {
  _currentAccount: ?AccountToken;
  _expiresAt: ?Date;
  _fetchAttempt: number;
  _fetchRetryTimeout: ?TimeoutID;
  _fetch: (AccountToken) => Promise<AccountData>;
  _update: (?AccountData) => void;
  _watchers: Array<AccountFetchWatcher>;

  constructor(fetch: (AccountToken) => Promise<AccountData>, update: (?AccountData) => void) {
    this._fetch = fetch;
    this._update = update;
    this._watchers = [];
    this._fetchAttempt = 0;
  }

  fetch(accountToken: AccountToken, watcher?: AccountFetchWatcher) {
    // invalidate cache if account token has changed
    if (accountToken !== this._currentAccount) {
      this.invalidate();
      this._currentAccount = accountToken;
    }

    // Only fetch is value has expired
    if (this._isExpired()) {
      if (watcher) {
        this._watchers.push(watcher);
      }

      this._performFetch(accountToken);
    } else if (watcher) {
      watcher.onFinish();
    }
  }

  invalidate() {
    if (this._fetchRetryTimeout) {
      clearTimeout(this._fetchRetryTimeout);
      this._fetchRetryTimeout = null;
      this._fetchAttempt = 0;
    }

    this._expiresAt = null;
    this._update(null);
    this._notifyWatchers((watcher) => {
      watcher.onError(new Error('Cancelled'));
    });
  }

  _setValue(value: AccountData) {
    this._expiresAt = new Date(Date.now() + 60 * 1000); // 60s expiration
    this._update(value);
    this._notifyWatchers((watcher) => watcher.onFinish());
  }

  _isExpired() {
    return !this._expiresAt || this._expiresAt < new Date();
  }

  async _performFetch(accountToken: AccountToken) {
    try {
      // it's possible for invalidate() to be called or for a fetch for a different account token
      // to start before this fetch completes, so checking if the current account token is the one
      // used is necessary below.
      const accountData = await this._fetch(accountToken);

      if (this._currentAccount === accountToken) {
        this._setValue(accountData);
      }
    } catch (error) {
      if (this._currentAccount === accountToken) {
        this._handleFetchError(accountToken, error);
      }
    }
  }

  _handleFetchError(accountToken: AccountToken, error: any) {
    let shouldRetry = true;

    this._notifyWatchers((watcher) => {
      if (watcher.onError(error) === 'stop') {
        shouldRetry = false;
      }
    });

    if (shouldRetry) {
      this._scheduleRetry(accountToken);
    }
  }

  _scheduleRetry(accountToken: AccountToken) {
    this._fetchAttempt += 1;

    const delay = Math.min(2048, 1 << (this._fetchAttempt + 2)) * 1000;

    log.debug(`Failed to fetch account data. Retrying in ${delay} ms`);

    this._fetchRetryTimeout = setTimeout(() => {
      this._fetchRetryTimeout = null;
      this._performFetch(accountToken);
    }, delay);
  }

  _notifyWatchers(notify: (AccountFetchWatcher) => void) {
    this._watchers.splice(0).forEach(notify);
  }
}
