import { closeToExpiry } from '../shared/account-expiry';
import {
  AccountToken,
  DeviceEvent,
  DeviceState,
  IAccountData,
  IDeviceRemoval,
  TunnelState,
} from '../shared/daemon-rpc-types';
import { messages } from '../shared/gettext';
import log from '../shared/logging';
import {
  AccountExpiredNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  SystemNotificationCategory,
} from '../shared/notifications/notification';
import { Scheduler } from '../shared/scheduler';
import AccountDataCache from './account-data-cache';
import { DaemonRpc } from './daemon-rpc';
import { InvalidAccountError } from './errors';
import { IpcMainEventChannel } from './ipc-event-channel';
import { NotificationSender } from './notification-controller';
import { TunnelStateProvider } from './tunnel-state';

export interface LocaleProvider {
  getLocale(): string;
}

export interface AccountDelegate {
  onDeviceEvent(): void;
}

export default class Account {
  private accountDataValue?: IAccountData = undefined;
  private accountHistoryValue?: AccountToken = undefined;
  private expiryNotificationFrequencyScheduler = new Scheduler();
  private firstExpiryNotificationScheduler = new Scheduler();

  private accountDataCache = new AccountDataCache(
    (accountToken) => {
      return this.daemonRpc.getAccountData(accountToken);
    },
    (accountData) => {
      this.accountDataValue = accountData;

      IpcMainEventChannel.account.notify?.(this.accountData);

      this.handleAccountExpiry();
    },
  );

  private deviceStateValue?: DeviceState;

  public constructor(
    private delegate: AccountDelegate & TunnelStateProvider & LocaleProvider & NotificationSender,
    private daemonRpc: DaemonRpc,
  ) {}

  public get accountData() {
    return this.accountDataValue;
  }

  public get accountHistory() {
    return this.accountHistoryValue;
  }

  public get deviceState() {
    return this.deviceStateValue;
  }

  public registerIpcListeners() {
    IpcMainEventChannel.account.handleCreate(() => this.createNewAccount());
    IpcMainEventChannel.account.handleLogin((token: AccountToken) => this.login(token));
    IpcMainEventChannel.account.handleLogout(() => this.logout());
    IpcMainEventChannel.account.handleGetWwwAuthToken(() => this.daemonRpc.getWwwAuthToken());
    IpcMainEventChannel.account.handleSubmitVoucher(async (voucherCode: string) => {
      const currentAccountToken = this.getAccountToken();
      const response = await this.daemonRpc.submitVoucher(voucherCode);

      if (currentAccountToken) {
        this.accountDataCache.handleVoucherResponse(currentAccountToken, response);
      }

      return response;
    });
    IpcMainEventChannel.account.handleUpdateData(() => this.updateAccountData());

    IpcMainEventChannel.accountHistory.handleClear(async () => {
      await this.daemonRpc.clearAccountHistory();
      void this.updateAccountHistory();
    });

    IpcMainEventChannel.account.handleGetDeviceState(async () => {
      try {
        await this.daemonRpc.updateDevice();
      } catch (e) {
        const error = e as Error;
        log.warn(`Failed to update device info: ${error.message}`);
      }
      return this.daemonRpc.getDevice();
    });
    IpcMainEventChannel.account.handleListDevices((accountToken: AccountToken) => {
      return this.daemonRpc.listDevices(accountToken);
    });
    IpcMainEventChannel.account.handleRemoveDevice((deviceRemoval: IDeviceRemoval) => {
      return this.daemonRpc.removeDevice(deviceRemoval);
    });
  }

  public isLoggedIn(): boolean {
    return this.deviceState?.type === 'logged in';
  }

  public updateAccountData = () => {
    if (this.daemonRpc.isConnected && this.isLoggedIn()) {
      this.accountDataCache.fetch(this.getAccountToken()!);
    }
  };

  public detectStaleAccountExpiry(tunnelState: TunnelState) {
    const hasExpired = !this.accountData || new Date() >= new Date(this.accountData.expiry);

    // It's likely that the account expiry is stale if the daemon managed to establish the tunnel.
    if (tunnelState.state === 'connected' && hasExpired) {
      log.info('Detected the stale account expiry.');
      this.accountDataCache.invalidate();
    }
  }

  public handleDeviceEvent(deviceEvent: DeviceEvent) {
    this.deviceStateValue = deviceEvent.deviceState;

    switch (deviceEvent.deviceState.type) {
      case 'logged in':
        this.accountDataCache.fetch(deviceEvent.deviceState.accountAndDevice.accountToken);
        break;
      case 'logged out':
      case 'revoked':
        this.accountDataCache.invalidate();
        break;
    }

    void this.updateAccountHistory();
    this.delegate.onDeviceEvent();

    IpcMainEventChannel.account.notifyDevice?.(deviceEvent);
  }

  public setAccountHistory(accountHistory?: AccountToken) {
    this.accountHistoryValue = accountHistory;

    IpcMainEventChannel.accountHistory.notify?.(accountHistory);
  }

  private async createNewAccount(): Promise<string> {
    try {
      return await this.daemonRpc.createNewAccount();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to create account: ${error.message}`);
      throw error;
    }
  }

  private async login(accountToken: AccountToken): Promise<void> {
    try {
      await this.daemonRpc.loginAccount(accountToken);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to login: ${error.message}`);

      if (error instanceof InvalidAccountError) {
        throw Error(messages.gettext('Invalid account number'));
      } else {
        throw error;
      }
    }
  }

  private async logout(): Promise<void> {
    try {
      await this.daemonRpc.logoutAccount();

      this.expiryNotificationFrequencyScheduler.cancel();
      this.firstExpiryNotificationScheduler.cancel();
    } catch (e) {
      const error = e as Error;
      log.info(`Failed to logout: ${error.message}`);

      throw error;
    }
  }

  private handleAccountExpiry() {
    if (this.accountData) {
      const expiredNotification = new AccountExpiredNotificationProvider({
        accountExpiry: this.accountData.expiry,
        tunnelState: this.delegate.getTunnelState(),
      });
      const closeToExpiryNotification = new CloseToAccountExpiryNotificationProvider({
        accountExpiry: this.accountData.expiry,
        locale: this.delegate.getLocale(),
      });

      if (expiredNotification.mayDisplay()) {
        this.expiryNotificationFrequencyScheduler.cancel();
        this.firstExpiryNotificationScheduler.cancel();
        this.delegate.notify(expiredNotification.getSystemNotification());
      } else if (
        !this.expiryNotificationFrequencyScheduler.isRunning &&
        closeToExpiryNotification.mayDisplay()
      ) {
        this.firstExpiryNotificationScheduler.cancel();
        this.delegate.notify(closeToExpiryNotification.getSystemNotification());

        const twelveHours = 12 * 60 * 60 * 1000;
        const remainingMilliseconds = new Date(this.accountData.expiry).getTime() - Date.now();
        const delay = Math.min(twelveHours, remainingMilliseconds);
        this.expiryNotificationFrequencyScheduler.schedule(() => this.handleAccountExpiry(), delay);
      } else if (!closeToExpiry(this.accountData.expiry)) {
        this.expiryNotificationFrequencyScheduler.cancel();
        // If no longer close to expiry, all previous notifications should be closed
        this.delegate.closeNotificationsInCategory(SystemNotificationCategory.expiry);

        const expiry = new Date(this.accountData.expiry).getTime();
        const now = new Date().getTime();
        const threeDays = 3 * 24 * 60 * 60 * 1000;
        // Add 10 seconds to be on the safe side. Never make it longer than a 24 days since
        // the timeout needs to fit into a signed 32-bit integer.
        const timeout = Math.min(expiry - now - threeDays + 10_000, 24 * 24 * 60 * 60 * 1000);
        this.firstExpiryNotificationScheduler.schedule(() => this.handleAccountExpiry(), timeout);
      }
    }
  }

  private async updateAccountHistory(): Promise<void> {
    try {
      this.setAccountHistory(await this.daemonRpc.getAccountHistory());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the account history: ${error.message}`);
    }
  }

  private getAccountToken(): AccountToken | undefined {
    return this.deviceState?.type === 'logged in'
      ? this.deviceState.accountAndDevice.accountToken
      : undefined;
  }
}
