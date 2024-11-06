import { expect, spy } from 'chai';
import sinon from 'sinon';

import AccountDataCache, { AccountFetchError } from '../../src/main/account-data-cache';
import { AccountDataResponse, IAccountData } from '../../src/shared/daemon-rpc-types';

describe('IAccountData cache', () => {
  const dummyAccountNumber = '9876543210';
  const dummyAccountData: AccountDataResponse = {
    type: 'success',
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
      (_number) => Promise.resolve(dummyAccountData),
      (_data) => {},
    );

    const watcher = new Promise<void>((resolve, reject) => {
      cache.fetch(dummyAccountNumber, {
        onFinish: () => resolve(),
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(watcher).to.eventually.be.fulfilled;
  });

  it('should notify when fetch fails on the first attempt', async () => {
    const cache = new AccountDataCache(
      (_number) => Promise.resolve({ type: 'error', error: 'invalid-account' }),
      (_data) => {},
    );

    const watcher = new Promise<void>((resolve, reject) => {
      cache.fetch(dummyAccountNumber, {
        onFinish: () => resolve(),
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(watcher).to.eventually.be.rejected;
  });

  it('should update when fetch succeeds on the first attempt', async () => {
    const update = new Promise<void>((resolve, reject) => {
      const cache = new AccountDataCache(
        (_) => Promise.resolve(dummyAccountData),
        () => resolve(),
      );

      cache.fetch(dummyAccountNumber, {
        onFinish: () => {},
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should update when fetch succeeds on the second attempt', async () => {
    const update = new Promise<void>((resolve, reject) => {
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

      cache.fetch(dummyAccountNumber, {
        onFinish: () => reject(),
        onError: (_error: AccountFetchError) => {},
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should cancel first fetch', async () => {
    const firstError = spy((_error: AccountFetchError) => {});
    const secondSuccess = spy();

    const update = new Promise<IAccountData | void>((resolve, reject) => {
      let firstAttempt = true;
      const fetch = (_number: string) => {
        if (firstAttempt) {
          firstAttempt = false;

          cache.fetch('1231231231', {
            onFinish: secondSuccess,
            onError: () => reject(),
          });

          return new Promise<AccountDataResponse>((resolve) => {
            setTimeout(() => resolve(dummyAccountData), 1000);
          });
        } else {
          reject();
          return Promise.resolve(dummyAccountData);
        }
      };

      const cache = new AccountDataCache(fetch, (_accountData?: IAccountData) => {
        resolve();
      });

      setTimeout(resolve, 12000);

      cache.fetch(dummyAccountNumber, {
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

  it('should clear scheduled retry if another fetch is performed', async () => {
    const firstError = spy();
    const secondSuccess = spy();
    const updateHandler = spy();

    const update = new Promise((resolve, reject) => {
      let attempts = 0;
      const fetch = (): Promise<AccountDataResponse> => {
        attempts++;
        if (attempts === 1) {
          return Promise.resolve({ type: 'error', error: 'invalid-account' });
        } else if (attempts === 2) {
          setTimeout(() => clock.tick(8000));
          return Promise.resolve(dummyAccountData);
        } else {
          reject();
          return Promise.resolve(dummyAccountData);
        }
      };

      const cache = new AccountDataCache(fetch, updateHandler);

      cache.fetch(dummyAccountNumber, {
        onFinish: () => {},
        onError: (_error: AccountFetchError) => firstError(),
      });
      setTimeout(() => {
        cache.fetch(dummyAccountNumber, {
          onFinish: () => {
            secondSuccess();
            setTimeout(resolve);
          },
          onError: (_error: AccountFetchError) => {},
        });
      });
    });

    return expect(update).to.eventually.be.fulfilled.then(() => {
      expect(firstError).to.have.been.called.once;
      expect(secondSuccess).to.have.been.called.once;
      expect(updateHandler).to.have.been.called.twice;
    });
  });

  it('should not perform a fetch if called twice synchronously', async () => {
    const fetchSpy = spy();
    const update = new Promise<void>((resolve, _reject) => {
      const fetch = () => {
        fetchSpy();
        return Promise.resolve(dummyAccountData);
      };

      const cache = new AccountDataCache(fetch, () => {});
      const onError = (_error: AccountFetchError) => {};
      cache.fetch(dummyAccountNumber, { onFinish: () => {}, onError });
      cache.fetch(dummyAccountNumber, { onFinish: () => resolve(), onError });
    });

    return expect(update).to.eventually.be.fulfilled.then(() => {
      expect(fetchSpy).to.have.been.called.once;
    });
  });

  it('should refetch one minute before expiry', async () => {
    const date = new Date();
    date.setMinutes(date.getMinutes() + 3);
    const expiry = date.toISOString();

    const update = new Promise<void>((resolve, reject) => {
      let firstAttempt = true;
      const fetch = (_accountNumber: string): Promise<AccountDataResponse> => {
        if (firstAttempt) {
          firstAttempt = false;
          setTimeout(() => clock.tick(120_000), 0);
          return Promise.resolve({ type: 'success', expiry });
        } else {
          resolve();
          return Promise.resolve({ type: 'success', expiry });
        }
      };

      const cache = new AccountDataCache(fetch, () => {});

      cache.fetch(dummyAccountNumber, {
        onFinish: () => {},
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(update).to.eventually.be.fulfilled;
  });

  it('should invalidate after 60 seconds', async () => {
    const fetchSpy = spy();
    const expiry = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();

    const update = new Promise<void>((resolve, reject) => {
      const cache = new AccountDataCache(
        (_accountNumber) => {
          fetchSpy();
          return Promise.resolve<AccountDataResponse>({ type: 'success', expiry });
        },
        () => {},
      );

      cache.fetch(dummyAccountNumber, {
        onFinish: async () => {
          clock.tick(59_000);
          // Timeout to let asynchronous tasks finish
          await new Promise((resolve) => setTimeout(resolve));

          cache.fetch(dummyAccountNumber, {
            onFinish: async () => {
              clock.tick(1_000);
              // Timeout to let asynchronous tasks finish
              await new Promise((resolve) => setTimeout(resolve));

              cache.fetch(dummyAccountNumber, {
                onFinish: () => resolve(),
                onError: (_error: AccountFetchError) => reject(),
              });
            },
            onError: (_error: AccountFetchError) => reject(),
          });
        },
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(update).to.eventually.be.fulfilled.then(() => {
      expect(fetchSpy).to.have.been.called.twice;
    });
  });

  it('should invalidate after 10 seconds when epired', async () => {
    const fetchSpy = spy();
    const expiry = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();

    const update = new Promise<void>((resolve, reject) => {
      const cache = new AccountDataCache(
        (_accountNumber) => {
          fetchSpy();
          return Promise.resolve<AccountDataResponse>({ type: 'success', expiry });
        },
        () => {},
      );

      cache.fetch(dummyAccountNumber, {
        onFinish: async () => {
          clock.tick(9_000);
          // Timeout to let asynchronous tasks finish
          await new Promise((resolve) => setTimeout(resolve));

          cache.fetch(dummyAccountNumber, {
            onFinish: async () => {
              clock.tick(1_000);
              // Timeout to let asynchronous tasks finish
              await new Promise((resolve) => setTimeout(resolve));

              cache.fetch(dummyAccountNumber, {
                onFinish: () => resolve(),
                onError: (_error: AccountFetchError) => reject(),
              });
            },
            onError: (_error: AccountFetchError) => reject(),
          });
        },
        onError: (_error: AccountFetchError) => reject(),
      });
    });

    return expect(update).to.eventually.be.fulfilled.then(() => {
      expect(fetchSpy).to.have.been.called.twice;
    });
  });
});
