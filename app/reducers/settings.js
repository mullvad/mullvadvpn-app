// @flow
import { handleActions } from 'redux-actions';
import { defaultServer } from '../config';
import actions from '../actions/settings';

import type { ReduxAction } from '../store';

export type SettingsReduxState = {
  autoSecure: boolean,
  preferredServer: string
};

const initialState: SettingsReduxState = {
  autoSecure: true,
  preferredServer: defaultServer
};

export default handleActions({
  [actions.updateSettings.toString()]: (state: SettingsReduxState, action: ReduxAction<$Shape<SettingsReduxState>>) => {
    return { ...state, ...action.payload };
  }
}, initialState);
