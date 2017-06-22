// @flow
import { createAction } from 'redux-actions';

import type { SettingsReduxState } from '../reducers/settings';
import type { ReduxAction } from '../store';

export type UpdateSettingsAction = <T: $Shape<SettingsReduxState>>(state: T) => ReduxAction<T>;

const updateSettings: UpdateSettingsAction = createAction('SETTINGS_UPDATE');

export default { updateSettings };
