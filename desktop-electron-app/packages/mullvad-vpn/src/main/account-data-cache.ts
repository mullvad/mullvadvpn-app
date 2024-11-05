import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import {
  AccountDataError,
  AccountDataResponse,
  AccountNumber,
  IAccountData,
  VoucherResponse,
} from '../shared/daemon-rpc-types';
import { dateByAddingComponent, DateComponent } from '../shared/date-helper';
import log from '../shared/logging';
import { Scheduler } from '../shared/scheduler';

export type AccountFetchError = AccountDataError['error'] | 'cancelled';

interface IAccountFetchWatcher {
  onFinish: () => void;
  onError: (error: AccountFetchError) => void;
}

// Account data is valid for 1 minute unless the account has expired.
const ACCOUNT_DATA_VALIDITY_SECONDS = 60_000;
// Account data is valid for 10 seconds if the account has expired.
const ACCOUNT_DATA_EXPIRED_VALIDITY_SECONDS = 10_000;

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
export default class AccountDataCache {
  private currentAccount?: AccountNumber;
  private validUntil?: Date;
  private performingFetch = false;
  private waitStrategy = new WaitStrategy();
  private fetchRetryScheduler = new Scheduler();
  private watchers: IAccountFetchWatcher[] = [];

  constructor(
    private fetchHandler: (number: AccountNumber) => Promise<AccountDataResponse>,
    private updateHandler: (data?: IAccountData) => void,
  ) {}

  public fetch(accountNumber: AccountNumber, watcher?: IAccountFetchWatcher) {
    // invalidate cache if account number has changed
    if (accountNumber !== this.currentAccount) {
      this.invalidate();
      this.currentAccount = accountNumber;
    }

    // Only fetch if value has expired
    if (!this.isValid()) {
      if (watcher) {
        this.watchers.push(watcher);
      }

      this.fetchRetryScheduler.cancel();
      // If a scheduled retry is cancelled the fetchAttempt shouldn't be increased.
      this.waitStrategy.decrease();

      // Only fetch if there's no fetch for this account number in progress.
      if (!this.performingFetch) {
        void this.performFetch(accountNumber);
      }
    } else if (watcher) {
      watcher.onFinish();
    }
  }

  public invalidate() {
    this.fetchRetryScheduler.cancel();
    this.waitStrategy.reset();

    this.performingFetch = false;
    this.validUntil = undefined;
    this.updateHandler();
    this.notifyWatchers((watcher) => {
      watcher.onError('cancelled');
    });
  }

  public handleVoucherResponse(accountNumber: AccountNumber, voucherResponse: VoucherResponse) {
    if (accountNumber === this.currentAccount && voucherResponse.type === 'success') {
      this.setValue({ expiry: voucherResponse.newExpiry });
    }
  }

  private setValue(accountData: IAccountData) {
    this.validUntil = this.getValidUntil(accountData);
    this.updateHandler(accountData);
    this.notifyWatchers((watcher) => watcher.onFinish());
  }

  private isValid() {
    return this.validUntil && this.validUntil > new Date();
  }

  private getValidUntil(accountData: IAccountData): Date {
    if (hasExpired(accountData.expiry)) {
      return new Date(Date.now() + ACCOUNT_DATA_EXPIRED_VALIDITY_SECONDS);
    } else {
      return new Date(Date.now() + ACCOUNT_DATA_VALIDITY_SECONDS);
    }
  }

  private async performFetch(accountNumber: AccountNumber) {
    this.performingFetch = true;
    // it's possible for invalidate() to be called or for a fetch for a different account number
    // to start before this fetch completes, so checking if the current account number is the one
    // used is necessary below.
    const response = await this.fetchHandler(accountNumber);
    if ('error' in response) {
      if (this.currentAccount === accountNumber) {
        this.handleFetchError(accountNumber, response.error);
        this.performingFetch = false;
      }
    } else {
      if (this.currentAccount === accountNumber) {
        this.setValue(response);

        const refetchDelay = this.calculateRefetchDelay(response.expiry);
        if (refetchDelay) {
          this.scheduleFetch(accountNumber, refetchDelay);
        }

        this.waitStrategy.reset();
        this.performingFetch = false;
      }
    }
  }

  private calculateRefetchDelay(accountExpiry: string) {
    const currentDate = new Date();
    const oneMinuteBeforeExpiry = dateByAddingComponent(accountExpiry, DateComponent.minute, -1);

    if (oneMinuteBeforeExpiry >= currentDate && closeToExpiry(accountExpiry)) {
      return oneMinuteBeforeExpiry.getTime() - currentDate.getTime();
    } else {
      return undefined;
    }
  }

  private handleFetchError(accountNumber: AccountNumber, error: AccountDataError['error']) {
    this.notifyWatchers((w) => w.onError(error));
    if (error !== 'invalid-account') {
      this.scheduleRetry(accountNumber);
    }
  }

  private scheduleRetry(accountNumber: AccountNumber) {
    this.waitStrategy.increase();
    const delay = this.waitStrategy.delay();

    log.warn(`Failed to fetch account data. Retrying in ${delay} ms`);

    this.scheduleFetch(accountNumber, delay);
  }

  private scheduleFetch(accountNumber: AccountNumber, delay: number) {
    this.fetchRetryScheduler.schedule(() => {
      void this.performFetch(accountNumber);
    }, delay);
  }

  private notifyWatchers(notify: (watcher: IAccountFetchWatcher) => void) {
    this.watchers.splice(0).forEach(notify);
  }
}

const MAX_ATTEMPT = 9;

class WaitStrategy {
  private counter = 0;

  public increase() {
    if (this.counter < MAX_ATTEMPT) {
      this.counter += 1;
    }
  }
  public decrease() {
    if (this.counter > 0) {
      this.counter -= 1;
    }
  }

  public reset() {
    this.counter = 0;
  }

  public delay(): number {
    // Max delay: 2^11 = 2048
    return Math.pow(2, this.counter + 2) * 1000;
  }
}
