// @flow

import { Backend } from '../../app/lib/backend';
import { newMockIpc } from '../mocks/ipc';
import configureStore from '../../app/redux/store';
import { createMemoryHistory } from 'history';
import { mockStore } from '../mocks/redux';

type DoneCallback = (?Error) => void;
type Check = () => void;

export function setupIpcAndStore() {
  const memoryHistory = createMemoryHistory();
  const store = configureStore(null, memoryHistory);

  const mockIpc = newMockIpc();

  return { store, mockIpc };
}

export function setupBackendAndStore() {

  const { store, mockIpc } = setupIpcAndStore();

  const credentials = {
    sharedSecret: '',
    connectionString: '',
  };
  const backend = new Backend(store, credentials, mockIpc);

  return { store, mockIpc, backend };
}

export function setupBackendAndMockStore() {
  const store = mockStore(_initialState());
  const mockIpc = newMockIpc();
  const credentials = {
    sharedSecret: '',
    connectionString: '',
  };
  const backend = new Backend(store, credentials, mockIpc);
  return { store, mockIpc, backend };
}

function _initialState() {
  const { store } = setupIpcAndStore();
  return store.getState();
}

// chai and async aren't the best of friends. To allow us
// to get the assertion error in the output of failed async
// tests we need to do this try-catch thing.
export function check(fn: Check, done: DoneCallback) {
  try {
    fn();
    done();
  } catch (e) {
    done(e);
  }
}

// Sometimes with redux we cannot know when all reducers have
// finished running. This function puts the check at the end
// of the execution queue, hopefully resulting in the check being
// run after the reducers are finished
export function checkNextTick(fn: Check, done: DoneCallback) {
  setTimeout(() => {
    check(fn, done);
  }, 1);
}


// In async tests where we want to test a chain of IPC messages
// we can only invoke `done` for the last message. This function
// is for the intermediate messages.
export function failFast(fn: Check, done: DoneCallback): boolean {
  try {
    fn();
    return false;
  } catch(e) {
    done(e);
    return true;
  }
}
export function failFastNextTick(fn: Check, done: DoneCallback) {
  setTimeout(() => {
    failFast(fn, done);
  }, 1);
}

type MockStore = {
  getActions: () => Array<{type: string, payload: Object}>,
}
// Parses the action log to find out which URL we most recently navigated to
// Note that this cannot be done with the real redux store, but rather must be
// done with the mock store.
export function getLocation(store: MockStore): ?string {
  const navigations = store.getActions().filter(action => action.type === '@@router/CALL_HISTORY_METHOD');
  if (navigations.length === 0) {
    return null;
  }

  return navigations[navigations.length - 1].payload.args[0];
}

