// @flow

import { expect } from 'chai';
import settingsReducer from '../app/redux/settings/reducers';
import { defaultServer } from '../app/config';

describe('reducers', () => {
  const previousState: any = {};

  it('should handle SETTINGS_UPDATE', () => {
    const action = {
      type: 'UPDATE_SETTINGS',
      newSettings: {
        preferredServer: defaultServer
      }
    };
    const test = Object.assign({}, action.newSettings);
    expect(settingsReducer(previousState, action)).to.deep.equal(test);
  });

});
