// @flow
import { resolveBin } from './proc';
import { execFile } from 'child_process';
import { ipcRenderer } from 'electron';
import { log } from './platform';
import uuid from 'uuid';

const collectProblemReport = (toRedact: Array<string>): Promise<string> => {
  return new Promise((resolve, reject) => {
    const requestId = uuid.v4();
    let responseListener: Function;

    const removeResponseListener = () => {
      ipcRenderer.removeListener('collect-logs-reply', responseListener);
    };

    // timeout after 10 seconds if no ipc response received
    const requestTimeout = setTimeout(() => {
      removeResponseListener();
      log.error('Timed out when collecting a problem report');
      reject(new Error('Timed out'));
    }, 10000);

    responseListener = (_event, id, error, reportPath) => {
      if (id !== requestId) {
        return;
      }

      clearTimeout(requestTimeout);
      removeResponseListener();

      if (error) {
        log.error(`Cannot collect a problem report: ${error.err}`);
        log.error(`Stdout: ${error.stdout}`);
        reject(error);
      } else {
        resolve(reportPath);
      }
    };

    // add ipc response listener
    ipcRenderer.on('collect-logs-reply', responseListener);

    // send ipc request
    ipcRenderer.send('collect-logs', requestId, toRedact);
  });
};

const sendProblemReport = (email: string, message: string, savedReport: string) => {
  const args = ['send', '--email', email, '--message', message, '--report', savedReport];

  const binPath = resolveBin('problem-report');

  return new Promise((resolve, reject) => {
    execFile(binPath, args, { windowsHide: true }, (err, stdout, stderr) => {
      if (err) {
        reject({ err, stdout, stderr });
      } else {
        log.debug('Report sent');
        resolve();
      }
    });
  }).catch((e) => {
    const { err, stdout } = e;
    log.error('Failed sending problem report', err);
    log.error('  stdout: ' + stdout);

    throw e;
  });
};

export { collectProblemReport, sendProblemReport };
