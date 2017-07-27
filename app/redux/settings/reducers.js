// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';

export type SettingsReduxState = {
  autoSecure: boolean,
  preferredServer: string
};

const initialState: SettingsReduxState = {
  autoSecure: true,
  preferredServer: defaultServer
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  if (action.type === 'UPDATE_SETTINGS') {
    return { ...state, ...action.newSettings };
  }

  return state;
}
