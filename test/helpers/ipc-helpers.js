// @flow

import { Backend } from '../../app/lib/backend';
import { newMockIpc } from '../mocks/ipc';
import configureStore from '../../app/redux/store';
import { createMemoryHistory } from 'history';

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

