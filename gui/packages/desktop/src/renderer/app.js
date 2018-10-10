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

import { InvalidAccountError } from './errors';
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
  RelayList,
  RelaySettingsUpdate,
  RelaySettings,
  TunnelState,
  DaemonRpcProtocol,
  AccountData,
} from './lib/daemon-rpc';
import type { ReduxStore } from './redux/store';
import type { TrayIconType } from '../main/tray-icon-controller';

const RELAY_LIST_UPDATE_INTERVAL = 60 * 60 * 1000;

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
  _accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this._daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      const expiry = accountData ? accountData.expiry : null;
      this._reduxActions.account.updateAccountExpiry(expiry);
    },
  );
  _relayListCache = new RelayListCache(
    () => {
      return this._daemonRpc.getRelayLocations();
    },
    (relayList) => {
      this._updateRelayLocations(relayList);
    },
  );
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

    ipcRenderer.on('window-shown', () => this.updateAccountExpiry());

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
        onError: (error) => {
          if (error instanceof InvalidAccountError) {
            reject(error);
          } else {
            resolve({ status: 'deferred', error });
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
    const settings = await this._settingsProxy.fetch();
    const accountDataCache = this._accountDataCache;

    if (settings && settings.accountToken) {
      accountDataCache.fetch(settings.accountToken);
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

  _updateRelayLocations(relayList: RelayList) {
    const actions = this._reduxActions;

    log.info('Got relay locations');

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

    actions.settings.updateRelayLocations(locations);
  }

  async _fetchLocation() {
    this._reduxActions.connection.newLocation(await this._daemonRpc.getLocation());
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

  async _getAppComponentsVersions() {
    const daemonVersion = await this._daemonRpc.getCurrentVersion();
    const guiVersion = remote.app.getVersion().replace('.0', '');
    return {
      daemon: daemonVersion,
      gui: guiVersion,
      isConsistent: daemonVersion === guiVersion,
    };
  }

  async _fetchCurrentVersion() {
    const actions = this._reduxActions;
    const versions = await this._getAppComponentsVersions();

    // notify user about inconsistent version
    if (process.env.NODE_ENV !== 'development' && !versions.isConsistent) {
      this._notificationController.notifyInconsistentVersion();
    }

    actions.version.updateVersion(versions.gui, versions.isConsistent);
  }

  async _fetchLatestVersionInfo() {
    function isBeta(version: string) {
      return version.includes('-');
    }

    function nextUpgrade(current: string, latest: string, latestStable: string): ?string {
      if (isBeta(current)) {
        return current === latest ? null : latest;
      } else {
        return current === latestStable ? null : latestStable;
      }
    }

    function checkIfLatest(current: string, latest: string, latestStable: string): boolean {
      // perhaps -beta?
      if (isBeta(current)) {
        return current === latest;
      } else {
        // must be stable
        return current === latestStable;
      }
    }

    const versions = await this._getAppComponentsVersions();
    const versionInfo = await this._daemonRpc.getVersionInfo();
    const latestVersion = versionInfo.latest.latest;
    const latestStableVersion = versionInfo.latest.latestStable;

    // the reason why we rely on daemon version here is because daemon obtains the version info
    // based on its built-in version information
    const isUpToDate = checkIfLatest(versions.daemon, latestVersion, latestStableVersion);
    const upgradeVersion = nextUpgrade(versions.daemon, latestVersion, latestStableVersion);

    // notify user to update the app if it became unsupported
    if (
      process.env.NODE_ENV !== 'development' &&
      versions.isConsistent &&
      !versionInfo.currentIsSupported &&
      upgradeVersion
    ) {
      this._notificationController.notifyUnsupportedVersion(upgradeVersion);
    }

    this._reduxActions.version.updateLatest({
      ...versionInfo,
      nextUpgrade: upgradeVersion,
      upToDate: isUpToDate,
    });
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
      await this._fetchCurrentVersion();
    } catch (error) {
      log.error(`Cannot fetch the current version: ${error.message}`);
    }

    this._relayListCache.startUpdating();

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
      await this._fetchLatestVersionInfo();
    } catch (error) {
      log.error(`Cannot fetch the latest version information: ${error.message}`);
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

  _onCloseConnection(error: ?Error) {
    const actions = this._reduxActions;

    this._relayListCache.stopUpdating();

    // recover connection on error
    if (error) {
      log.debug(`Lost connection to daemon: ${error.message}`);

      this._reconnectBackoff.attempt(() => {
        this.connect();
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
    this._notificationController.notifyTunnelState(tunnelState);
  }

  _setSettings(newSettings: Settings) {
    const reduxSettings = this._reduxActions.settings;

    reduxSettings.updateAllowLan(newSettings.allowLan);
    reduxSettings.updateAutoConnect(newSettings.autoConnect);
    reduxSettings.updateEnableIpv6(newSettings.tunnelOptions.enableIpv6);
    reduxSettings.updateOpenVpnMssfix(newSettings.tunnelOptions.openvpn.mssfix);

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
        actions.connection.disconnecting(stateTransition.details);
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
}

type AccountVerification = { status: 'verified' } | { status: 'deferred', error: Error };
type AccountFetchWatcher = {
  onFinish: () => void,
  onError: (any) => void,
};

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
class AccountDataCache {
  _currentAccount: ?AccountToken;
  _expiresAt: ?Date;
  _fetch: (AccountToken) => Promise<AccountData>;
  _update: (?AccountData) => void;
  _watchers: Array<AccountFetchWatcher>;

  constructor(fetch: (AccountToken) => Promise<AccountData>, update: (?AccountData) => void) {
    this._fetch = fetch;
    this._update = update;
    this._watchers = [];
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
    this._expiresAt = null;
    this._update(null);
    this._notifyWatchers((watcher) => watcher.onError(new Error('Cancelled')));
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
        this._notifyWatchers((watcher) => watcher.onError(error));
      }
  }

  _notifyWatchers(notify: (AccountFetchWatcher) => void) {
    this._watchers.splice(0).forEach(notify);
  }
}

class RelayListCache {
  _fetch: () => Promise<RelayList>;
  _listener: (RelayList) => void;
  _updateTimer: ?IntervalID = null;

  constructor(fetch: () => Promise<RelayList>, listener: (RelayList) => void) {
    this._fetch = fetch;
    this._listener = listener;
  }

  startUpdating() {
    this.stopUpdating();
    this._updateTimer = setInterval(() => this._update(), RELAY_LIST_UPDATE_INTERVAL);
    this._update();
  }

  stopUpdating() {
    if (this._updateTimer) {
      clearInterval(this._updateTimer);
    }
  }

  async _update() {
    try {
      this._listener(await this._fetch());
    } catch (error) {
      log.error(`Cannot fetch the relay locations: ${error.message}`);
    }
  }
}

function getIpcPath(): string {
  if (process.platform === 'win32') {
    return '//./pipe/Mullvad VPN';
  } else {
    return '/var/run/mullvad-vpn';
  }
}
