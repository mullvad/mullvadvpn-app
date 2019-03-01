import { ReduxAction } from '../store';

export interface ISupportReduxState {
  email: string;
  message: string;
}

const initialState: ISupportReduxState = {
  email: '',
  message: '',
};

export default function(
  state: ISupportReduxState = initialState,
  action: ReduxAction,
): ISupportReduxState {
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
