// @flow

import type { SettingsReduxState } from './reducers';

export type UpdateSettingsAction = {
  type: 'UPDATE_SETTINGS',
  newSettings: $Shape<SettingsReduxState>,
};

function updateSettings(newSettings: $Shape<SettingsReduxState>): UpdateSettingsAction {
  return {
    type: 'UPDATE_SETTINGS',
    newSettings: newSettings,
  };
}

export default { updateSettings };
