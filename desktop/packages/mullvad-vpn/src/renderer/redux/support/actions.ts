export interface IProblemReportForm {
  email: string;
  message: string;
}

export interface IKeepReportFormAction {
  type: 'SAVE_REPORT_FORM';
  form: IProblemReportForm;
}

export interface IClearReportFormAction {
  type: 'CLEAR_REPORT_FORM';
}

export type SupportAction = IKeepReportFormAction | IClearReportFormAction;

function saveReportForm(form: IProblemReportForm): IKeepReportFormAction {
  return {
    type: 'SAVE_REPORT_FORM',
    form,
  };
}

function clearReportForm(): IClearReportFormAction {
  return {
    type: 'CLEAR_REPORT_FORM',
  };
}

export default { saveReportForm, clearReportForm };
