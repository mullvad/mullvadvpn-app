import log from 'electron-log';
import moment from 'moment';
import { AccountToken, IAccountData } from '../shared/daemon-rpc-types';
import consumePromise from '../shared/promise';

const EXPIRED_ACCOUNT_REFRESH_PERIOD = 60_000;

export enum AccountFetchRetryAction {
  stop,
  retry,
}
interface IAccountFetchWatcher {
  onFinish: () => void;
  onError: (error: Error) => AccountFetchRetryAction;
}

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
export default class AccountDataCache {
  private currentAccount?: AccountToken;
  private expiresAt?: Date;
  private fetchAttempt = 0;
  private fetchRetryTimeout?: NodeJS.Timeout;
  private watchers: IAccountFetchWatcher[] = [];

  constructor(
    private fetchHandler: (token: AccountToken) => Promise<IAccountData>,
    private updateHandler: (data?: IAccountData) => void,
  ) {}

  public fetch(accountToken: AccountToken, watcher?: IAccountFetchWatcher) {
    // invalidate cache if account token has changed
    if (accountToken !== this.currentAccount) {
      this.invalidate();
      this.currentAccount = accountToken;
    }

    // Only fetch is value has expired
    if (this.isExpired()) {
      if (watcher) {
        this.watchers.push(watcher);
      }

      consumePromise(this.performFetch(accountToken));
    } else if (watcher) {
      watcher.onFinish();
    }
  }

  public invalidate() {
    if (this.fetchRetryTimeout) {
      clearTimeout(this.fetchRetryTimeout);
      this.fetchRetryTimeout = undefined;
      this.fetchAttempt = 0;
    }

    this.expiresAt = undefined;
    this.updateHandler();
    this.notifyWatchers((watcher) => {
      watcher.onError(new Error('Cancelled'));
    });
  }

  private setValue(value: IAccountData) {
    this.expiresAt = new Date(Date.now() + 60 * 1000); // 60s expiration
    this.updateHandler(value);
    this.notifyWatchers((watcher) => watcher.onFinish());
  }

  private isExpired() {
    return !this.expiresAt || this.expiresAt < new Date();
  }

  private async performFetch(accountToken: AccountToken) {
    try {
      // it's possible for invalidate() to be called or for a fetch for a different account token
      // to start before this fetch completes, so checking if the current account token is the one
      // used is necessary below.
      const accountData = await this.fetchHandler(accountToken);

      if (this.currentAccount === accountToken) {
        this.setValue(accountData);
        this.checkAccountExpiry(accountToken, accountData);
      }
    } catch (error) {
      if (this.currentAccount === accountToken) {
        this.handleFetchError(accountToken, error);
      }
    }
  }

  private checkAccountExpiry(accountToken: AccountToken, accountData: IAccountData) {
    const hasExpired = moment(accountData.expiry).isSameOrBefore(new Date());
    if (hasExpired) {
      this.scheduleFetch(accountToken, EXPIRED_ACCOUNT_REFRESH_PERIOD);
    }
  }

  private handleFetchError(accountToken: AccountToken, error: Error) {
    let shouldRetry = true;

    this.notifyWatchers((watcher) => {
      if (watcher.onError(error) === AccountFetchRetryAction.stop) {
        shouldRetry = false;
      }
    });

    if (shouldRetry) {
      this.scheduleRetry(accountToken);
    }
  }

  private scheduleRetry(accountToken: AccountToken) {
    this.fetchAttempt += 1;

    // Max delay: 2^11 = 2048
    const delay = Math.pow(2, Math.min(this.fetchAttempt + 2, 11)) * 1000;

    log.warn(`Failed to fetch account data. Retrying in ${delay} ms`);

    this.scheduleFetch(accountToken, delay);
  }

  private scheduleFetch(accountToken: AccountToken, delay: number) {
    this.fetchRetryTimeout = global.setTimeout(() => {
      this.fetchRetryTimeout = undefined;
      consumePromise(this.performFetch(accountToken));
    }, delay);
  }

  private notifyWatchers(notify: (watcher: IAccountFetchWatcher) => void) {
    this.watchers.splice(0).forEach(notify);
  }
}
