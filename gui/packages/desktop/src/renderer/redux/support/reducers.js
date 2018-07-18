// @flow

import type { ReduxAction } from '../store';

export type SupportReduxState = {
  email: string,
  message: string,
};

const initialState: SupportReduxState = {
  email: '',
  message: '',
};

export default function(
  state: SupportReduxState = initialState,
  action: ReduxAction,
): SupportReduxState {
  switch (action.type) {
    case 'SAVE_REPORT_FORM':
      return {
        ...state,
        email: action.form.email,
        message: action.form.message,
      };

    case 'CLEAR_REPORT_FORM':
      return {
        ...state,
        email: '',
        message: '',
      };

    default:
      return state;
  }
}
