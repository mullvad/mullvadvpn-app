// @flow

export type SupportReportForm = {
  email: string,
  message: string,
};

export type KeepReportFormAction = {
  type: 'SAVE_REPORT_FORM',
  form: SupportReportForm,
};

export type ClearReportFormAction = {
  type: 'CLEAR_REPORT_FORM',
};

export type SupportAction = KeepReportFormAction | ClearReportFormAction;

function saveReportForm(form: SupportReportForm): KeepReportFormAction {
  return {
    type: 'SAVE_REPORT_FORM',
    form,
  };
}

function clearReportForm(): ClearReportFormAction {
  return {
    type: 'CLEAR_REPORT_FORM',
  };
}

export default { saveReportForm, clearReportForm };
