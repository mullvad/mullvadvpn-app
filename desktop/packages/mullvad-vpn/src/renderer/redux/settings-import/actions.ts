export interface SaveSettingsImportFormAction {
  type: 'SAVE_SETTINGS_IMPORT_FORM';
  value: string;
  submit: boolean;
}

export interface ClearSettingsImportFormAction {
  type: 'CLEAR_SETTINGS_IMPORT_FORM';
}

export interface UnsetSubmitSettingsImportFormAction {
  type: 'UNSET_SUBMIT_SETTINGS_IMPORT_FORM';
}

export type SettingsImportAction =
  | SaveSettingsImportFormAction
  | ClearSettingsImportFormAction
  | UnsetSubmitSettingsImportFormAction;

function saveSettingsImportForm(value: string, submit: boolean): SaveSettingsImportFormAction {
  return {
    type: 'SAVE_SETTINGS_IMPORT_FORM',
    value,
    submit,
  };
}

function clearSettingsImportForm(): ClearSettingsImportFormAction {
  return {
    type: 'CLEAR_SETTINGS_IMPORT_FORM',
  };
}

function unsetSubmitSettingsImportForm(): UnsetSubmitSettingsImportFormAction {
  return {
    type: 'UNSET_SUBMIT_SETTINGS_IMPORT_FORM',
  };
}

export default { saveSettingsImportForm, clearSettingsImportForm, unsetSubmitSettingsImportForm };
