// @flow

import { expect } from 'chai';
import { createMemoryHistory } from 'history';
import configureStore from '../app/redux/store';
import connectionActions from '../app/redux/connection/actions';

describe('The connection state', () => {

  it('should contain the latest IP', () => {
    const memoryHistory = createMemoryHistory();
    const store = configureStore(null, memoryHistory);

    store.dispatch(connectionActions.newPublicIp('1.2.3.4'));
    store.dispatch(connectionActions.newPublicIp('5.6.7.8'));

    expect(store.getState().connection.clientIp).to.equal('5.6.7.8');
  });
});
