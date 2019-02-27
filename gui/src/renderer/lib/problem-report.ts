import { ipcRenderer } from 'electron';
import * as uuid from 'uuid';

interface IErrorResult {
  success: false;
  error: string;
}
type CollectResult = { success: true; reportPath: string } | IErrorResult;
type SendResult = { success: true } | IErrorResult;

const collectProblemReport = (toRedact: string[]): Promise<string> => {
  return new Promise((resolve, reject) => {
    const requestId = uuid.v4();
    const responseListener = (
      _event: Electron.Event,
      responseId: string,
      result: CollectResult,
    ) => {
      if (responseId === requestId) {
        ipcRenderer.removeListener('collect-logs-reply', responseListener);
        if (result.success) {
          resolve(result.reportPath);
        } else {
          reject(new Error(result.error));
        }
      }
    };

    ipcRenderer.on('collect-logs-reply', responseListener);
    ipcRenderer.send('collect-logs', requestId, toRedact);
  });
};

const sendProblemReport = (email: string, message: string, savedReport: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    const requestId = uuid.v4();
    const responseListener = (_event: Electron.Event, responseId: string, result: SendResult) => {
      if (requestId === responseId) {
        ipcRenderer.removeListener('send-problem-report-reply', responseListener);
        if (result.success) {
          resolve();
        } else {
          reject(new Error(result.error));
        }
      }
    };

    ipcRenderer.on('send-problem-report-reply', responseListener);
    ipcRenderer.send('send-problem-report', requestId, email, message, savedReport);
  });
};

export { collectProblemReport, sendProblemReport };
