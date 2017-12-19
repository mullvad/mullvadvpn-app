// @flow

import { defaultServer } from '../../config';

import type { ReduxAction } from '../store';

export type SettingsReduxState = {
  preferredServer: string
};

const initialState: SettingsReduxState = {
  preferredServer: defaultServer
};

export default function(state: SettingsReduxState = initialState, action: ReduxAction): SettingsReduxState {

  if (action.type === 'UPDATE_SETTINGS') {
    return { ...state, ...action.newSettings };
  }

  return state;
}
