import { AccountDataCache } from '../src/renderer/app';
import { AccountData } from '../src/shared/daemon-rpc-types';
import * as sinon from 'sinon';
import chai from 'chai';
import spies from 'chai-spies';
import chaiAsPromised from 'chai-as-promised';
import { it, describe, beforeEach, afterEach } from 'mocha';

const { expect, spy } = chai;

describe('AccountData cache', () => {
  const dummyAccountToken = '9876543210';
  const dummyAccountData: AccountData = {
    expiry: new Date('2038-01-01').toISOString(),
  };

  let clock: sinon.SinonFakeTimers;

  beforeEach(() => {
    clock = sinon.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    clock.restore();
  });

  it('should notify when fetch succeeds on the first attempt', async () => {
    const cache = new AccountDataCache(
      (_token) => Promise.resolve(dummyAccountData),
      (_data) => {},
    );

    const watcher = new Promise((resolve, reject) => {
      cache.fetch(dummyAccountToken, {
        onFinish: () => resolve(),
        onError: (_error: Error) => {
          reject();
          return 'stop';
        },
      });
    });

    return expect(watcher).to.eventually.be.fulfilled;
  });

  it('should notify when fetch fails on the first attempt', async () => {
    const cache = new AccountDataCache(
      (_token) => Promise.reject(new Error('Fetch fail')),
      (_data) => {},
    );

    const watcher = new Promise((resolve, reject) => {
      cache.fetch(dummyAccountToken, {
        onFinish: (_reason?: any) => resolve(),
        onError: (_error: Error) => {
          reject();
          return 'stop';
        },
      });
    });

    return expect(watcher).to.eventually.be.rejected;
  });

  it('should update when fetch succeeds on the first attempt', async () => {
    const update = new Promise((resolve, reject) => {
      const cache = new AccountDataCache((_) => Promise.resolve(dummyAccountData), () => resolve());

      cache.fetch(dummyAccountToken, {
        onFinish: spy(),
        onError: (_error: Error) => {
          reject();
          return 'stop';
        },
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should update when fetch succeeds on the second attempt', async () => {
    const update = new Promise((resolve, reject) => {
      let firstAttempt = true;
      const fetch = () => {
        if (firstAttempt) {
          firstAttempt = false;
          setTimeout(() => clock.tick(9000), 0);
          return Promise.reject(new Error('First attempt fails'));
        } else {
          resolve();
          return Promise.resolve(dummyAccountData);
        }
      };

      const cache = new AccountDataCache(fetch, () => resolve());

      cache.fetch(dummyAccountToken, {
        onFinish: () => reject(),
        onError: spy((_error: Error) => 'retry'),
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should not retry if told to stop', async () => {
    const update = new Promise((resolve, reject) => {
      let firstAttempt = true;
      const fetch = () => {
        if (firstAttempt) {
          firstAttempt = false;
          setTimeout(() => clock.tick(14000), 0);
          return Promise.reject(new Error('First attempt fails'));
        } else {
          reject();
          return Promise.resolve(dummyAccountData);
        }
      };

      const cache = new AccountDataCache(fetch, () => resolve());

      setTimeout(resolve, 12000);

      cache.fetch(dummyAccountToken, {
        onFinish: spy(),
        onError: spy((_error: Error) => 'stop'),
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should cancel first fetch', async () => {
    const firstError = spy((_) => 'stop');
    const secondSuccess = spy();

    const update = new Promise((resolve, reject) => {
      let firstAttempt = true;
      const fetch = () => {
        if (firstAttempt) {
          firstAttempt = false;

          cache.fetch('1231231231', {
            onFinish: secondSuccess,
            onError: () => {
              reject();
              return 'stop';
            },
          });

          return new Promise((resolve) => setTimeout(() => resolve(dummyAccountData), 1000));
        } else {
          reject();
          return Promise.resolve(dummyAccountData);
        }
      };

      const cache = new AccountDataCache(fetch, () => resolve());

      setTimeout(resolve, 12000);

      cache.fetch(dummyAccountToken, {
        onFinish: reject,
        onError: firstError,
      });
    });

    return expect(update).to.eventually.be.fulfilled.then(() => {
      expect(firstError).to.have.been.called.once;
      expect(secondSuccess).to.have.been.called.once;
      return;
    });
  });
});
