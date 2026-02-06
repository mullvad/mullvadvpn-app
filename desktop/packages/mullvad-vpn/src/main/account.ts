import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import {
  AccountDataError,
  AccountNumber,
  DeviceEvent,
  DeviceState,
  IAccountData,
  IDeviceRemoval,
  LogoutSource,
  TunnelState,
} from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import {
  AccountExpiredNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  SystemNotificationCategory,
} from '../shared/notifications';
import { Scheduler } from '../shared/scheduler';
import AccountDataCache from './account-data-cache';
import { DaemonRpc } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';
import { NotificationSender } from './notification-controller';
import { systemTimeMonitor } from './system-time-monitor';
import { TunnelStateProvider } from './tunnel-state';

export interface LocaleProvider {
  getLocale(): string;
}

export interface AccountDelegate {
  onDeviceEvent(): void;
}

export default class Account {
  private accountDataValue?: IAccountData = undefined;
  private accountHistoryValue?: AccountNumber = undefined;
  private expiryNotificationFrequencyScheduler = new Scheduler();
  private firstExpiryNotificationScheduler = new Scheduler();

  private hasExpired = false;

  private accountDataCache = new AccountDataCache(
    (accountNumber) => {
      return this.daemonRpc.getAccountData(accountNumber);
    },
    (accountData) => {
      this.handleAccountData(accountData);
    },
  );

  private deviceStateValue?: DeviceState;

  public constructor(
    private delegate: AccountDelegate & TunnelStateProvider & LocaleProvider & NotificationSender,
    private daemonRpc: DaemonRpc,
  ) {
    this.monitorExpiredChange();
  }

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
    IpcMainEventChannel.account.handleLogin(
      async (number: AccountNumber) => (await this.login(number)) ?? undefined,
    );
    IpcMainEventChannel.account.handleLogout((source) => this.logout(source));
    IpcMainEventChannel.account.handleGetWwwAuthToken(() => this.daemonRpc.getWwwAuthToken());
    IpcMainEventChannel.account.handleSubmitVoucher(async (voucherCode: string) => {
      const currentAccountNumber = this.getAccountNumber();
      const response = await this.daemonRpc.submitVoucher(voucherCode);

      if (currentAccountNumber) {
        this.accountDataCache.handleVoucherResponse(currentAccountNumber, response);
      }

      return response;
    });
    IpcMainEventChannel.account.handleUpdateData(() => this.updateAccountData());

    IpcMainEventChannel.accountHistory.handleClear(async () => {
      await this.daemonRpc.clearAccountHistory();
      void this.updateAccountHistory();
    });

    IpcMainEventChannel.account.handleListDevices((accountNumber: AccountNumber) => {
      return this.daemonRpc.listDevices(accountNumber);
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
      this.accountDataCache.fetch(this.getAccountNumber()!);
    }
  };

  public detectStaleAccountExpiry(tunnelState: TunnelState) {
    const expired = !this.accountData || hasExpired(this.accountData.expiry);

    // It's likely that the account expiry is stale if the daemon managed to establish the tunnel.
    if (tunnelState.state === 'connected' && expired) {
      log.info('Detected the stale account expiry.');
      this.accountDataCache.invalidate();
    }
  }

  public handleDeviceEvent(deviceEvent: DeviceEvent) {
    this.delegate.closeNotificationsInCategory(SystemNotificationCategory.expiry);

    this.deviceStateValue = deviceEvent.deviceState;

    void this.updateAccountHistory();
    this.delegate.onDeviceEvent();

    // When logging out the renderer process needs to receive the device update before the account
    // data update. This means that the ipc-call `account.notifyDevice` needs to be called before
    // invalidating the accountDateCache since that triggers the ipc-call `account.notify`.
    IpcMainEventChannel.account.notifyDevice?.(deviceEvent);

    switch (deviceEvent.deviceState.type) {
      case 'logged in':
        this.accountDataCache.fetch(deviceEvent.deviceState.accountAndDevice.accountNumber);
        break;
      case 'logged out':
      case 'revoked':
        this.accountDataCache.invalidate();
        break;
    }
  }

  public setAccountHistory(accountHistory?: AccountNumber) {
    this.accountHistoryValue = accountHistory;

    IpcMainEventChannel.accountHistory.notify?.(accountHistory);
  }

  // This function monitors if the account is expired due to system clock changes.
  private monitorExpiredChange() {
    systemTimeMonitor(() => {
      const expired = this.accountData && hasExpired(this.accountData.expiry);
      if (expired !== this.hasExpired) {
        this.handleAccountData(this.accountData);
      }
    });
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

  private async login(accountNumber: AccountNumber): Promise<AccountDataError | void> {
    const error = await this.daemonRpc.loginAccount(accountNumber);

    if (error) {
      log.error(`Failed to login: ${error.error}`);
      return error;
    }
  }

  private async logout(source: LogoutSource): Promise<void> {
    try {
      await this.daemonRpc.logoutAccount(source);

      this.delegate.closeNotificationsInCategory(SystemNotificationCategory.expiry);
      this.expiryNotificationFrequencyScheduler.cancel();
      this.firstExpiryNotificationScheduler.cancel();
    } catch (e) {
      const error = e as Error;
      log.info(`Failed to logout: ${error.message}`);

      throw error;
    }
  }

  private handleAccountData(accountData?: IAccountData) {
    this.accountDataValue = accountData;
    this.hasExpired = this.accountData !== undefined && hasExpired(this.accountData?.expiry);
    IpcMainEventChannel.account.notify?.(this.accountData);
    this.showNotifications();
  }

  private showNotifications() {
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
        this.expiryNotificationFrequencyScheduler.schedule(() => this.showNotifications(), delay);
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
        this.firstExpiryNotificationScheduler.schedule(() => this.showNotifications(), timeout);
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

  private getAccountNumber(): AccountNumber | undefined {
    return this.deviceState?.type === 'logged in'
      ? this.deviceState.accountAndDevice.accountNumber
      : undefined;
  }
}
