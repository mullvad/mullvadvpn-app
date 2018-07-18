// @flow

import { ipcRenderer } from 'electron';
import uuid from 'uuid';

const collectProblemReport = (toRedact: Array<string>): Promise<string> => {
  return new Promise((resolve, reject) => {
    const requestId = uuid.v4();
    const responseListener = (_event, responseId, result) => {
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
    const responseListener = (_event, responseId, result) => {
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
