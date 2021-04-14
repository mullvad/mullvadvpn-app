import { closeToExpiry, hasExpired } from '../shared/account-expiry';
import { AccountToken, IAccountData } from '../shared/daemon-rpc-types';
import { DateComponent, dateByAddingComponent } from '../shared/date-helper';
import log from '../shared/logging';
import consumePromise from '../shared/promise';
import { Scheduler } from '../shared/scheduler';
import { InvalidAccountError } from './errors';

const EXPIRED_ACCOUNT_REFRESH_PERIOD = 60_000;

interface IAccountFetchWatcher {
  onFinish: () => void;
  onError: (error: Error) => void;
}

// An account data cache that helps to throttle RPC requests to get_account_data and retain the
// cached value for 1 minute.
export default class AccountDataCache {
  private currentAccount?: AccountToken;
  private expiresAt?: Date;
  private performingFetch = false;
  private waitStrategy = new WaitStrategy();
  private fetchRetryScheduler = new Scheduler();
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

    // Only fetch if value has expired
    if (this.isExpired()) {
      if (watcher) {
        this.watchers.push(watcher);
      }

      this.fetchRetryScheduler.cancel();
      // If a scheduled retry is cancelled the fetchAttempt shouldn't be increased.
      this.waitStrategy.decrease();

      // Only fetch if there's no fetch for this account number in progress.
      if (!this.performingFetch) {
        consumePromise(this.performFetch(accountToken));
      }
    } else if (watcher) {
      watcher.onFinish();
    }
  }

  public invalidate() {
    this.fetchRetryScheduler.cancel();
    this.waitStrategy.reset();

    this.performingFetch = false;
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
    this.performingFetch = true;
    try {
      // it's possible for invalidate() to be called or for a fetch for a different account token
      // to start before this fetch completes, so checking if the current account token is the one
      // used is necessary below.
      const accountData = await this.fetchHandler(accountToken);

      if (this.currentAccount === accountToken) {
        this.setValue(accountData);

        const refetchDelay = this.calculateRefetchDelay(accountData.expiry);
        if (refetchDelay) {
          this.scheduleFetch(accountToken, refetchDelay);
        }

        this.waitStrategy.reset();
        this.performingFetch = false;
      }
    } catch (error) {
      if (this.currentAccount === accountToken) {
        this.handleFetchError(accountToken, error);
        this.performingFetch = false;
      }
    }
  }

  private calculateRefetchDelay(accountExpiry: string) {
    const currentDate = new Date();
    const oneMinuteBeforeExpiry = dateByAddingComponent(accountExpiry, DateComponent.minute, -1);

    if (hasExpired(accountExpiry)) {
      return EXPIRED_ACCOUNT_REFRESH_PERIOD;
    } else if (oneMinuteBeforeExpiry >= currentDate && closeToExpiry(accountExpiry)) {
      return oneMinuteBeforeExpiry.getTime() - currentDate.getTime();
    } else {
      return undefined;
    }
  }

  private handleFetchError(accountToken: AccountToken, error: Error) {
    this.notifyWatchers((w) => w.onError(error));
    if (!(error instanceof InvalidAccountError)) {
      this.scheduleRetry(accountToken);
    }
  }

  private scheduleRetry(accountToken: AccountToken) {
    this.waitStrategy.increase();
    const delay = this.waitStrategy.delay();

    log.warn(`Failed to fetch account data. Retrying in ${delay} ms`);

    this.scheduleFetch(accountToken, delay);
  }

  private scheduleFetch(accountToken: AccountToken, delay: number) {
    this.fetchRetryScheduler.schedule(() => {
      consumePromise(this.performFetch(accountToken));
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
