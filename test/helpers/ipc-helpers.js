// @flow

import { Backend } from '../../app/lib/backend';
import { newMockIpc } from '../mocks/ipc';
import configureStore from '../../app/redux/store';
import { createMemoryHistory } from 'history';
import { mockState, mockStore } from '../mocks/redux';

type DoneCallback = (?mixed) => void;
type Check = () => void;

// Mock localStorage because redux-localstorage has no test helpers
// We use redux-localstorage when we setup the redux store to have the
// store persist when the application is shut down.
global.localStorage = {getItem: ()=>'{}', setItem: ()=>{}};

export function setupBackendAndStore() {

  const memoryHistory = createMemoryHistory();
  const store = configureStore(null, memoryHistory);

  const mockIpc = newMockIpc();

  const backend = new Backend(store, mockIpc);

  return { store, mockIpc, backend };
}

export function setupBackendAndMockStore() {
  const store = mockStore(mockState());
  const mockIpc = newMockIpc();
  const backend = new Backend(store, mockIpc);
  return { store, mockIpc, backend };
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
export function failFast(fn: Check, done: DoneCallback) {
  try {
    fn();
  } catch(e) {
    done(e);
  }
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

