import { ReduxAction } from '../store';

export interface SettingsImportReduxState {
  value: string;
  submit: boolean;
}

const initialState: SettingsImportReduxState = {
  value: '',
  submit: false,
};

export default function (
  state: SettingsImportReduxState = initialState,
  action: ReduxAction,
): SettingsImportReduxState {
  switch (action.type) {
    case 'SAVE_SETTINGS_IMPORT_FORM':
      return {
        ...state,
        value: action.value,
        submit: action.submit,
      };

    case 'CLEAR_SETTINGS_IMPORT_FORM':
      return {
        ...state,
        value: '',
        submit: false,
      };

    case 'UNSET_SUBMIT_SETTINGS_IMPORT_FORM':
      return {
        ...state,
        submit: false,
      };

    default:
      return state;
  }
}
