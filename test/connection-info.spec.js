// @flow

import { expect } from 'chai';
import { createMemoryHistory } from 'history';
import configureStore from '../app/redux/store';
import connectionActions from '../app/redux/connection/actions';

describe('The connection state', () => {

  it('should contain the latest location', () => {
    const store = createStore();

    const firstLoc = {
      location: [1, 2],
      city: 'a',
      country: 'b',
    };
    const secondLoc = {
      location: [3, 4],
      city: 'c',
      country: 'd',
    };

    store.dispatch(connectionActions.newLocation(firstLoc));
    store.dispatch(connectionActions.newLocation(secondLoc));

    const { location, city, country } = store.getState().connection;
    expect(location).to.equal(secondLoc.location);
    expect(city).to.equal(secondLoc.city);
    expect(country).to.equal(secondLoc.country);
  });
});

function createStore() {
  const memoryHistory = createMemoryHistory();
  return configureStore(null, memoryHistory);
}
