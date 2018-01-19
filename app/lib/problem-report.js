// @flow
import { resolveBin } from './proc';
import { execFile } from 'child_process';
import { ipcRenderer } from 'electron';
import { log } from './platform';
import uuid from 'uuid';

const collectProblemReport = (toRedact: string) => {
  const unAnsweredIpcCalls = new Map();
  function reapIpcCall(id) {
    const promise = unAnsweredIpcCalls.get(id);
    unAnsweredIpcCalls.delete(id);

    if (promise) {
      promise.reject(new Error('Timed out'));
    }
  }
  ipcRenderer.on('collect-logs-reply', (_event, id, err, reportId) => {
    const promise = unAnsweredIpcCalls.get(id);
    unAnsweredIpcCalls.delete(id);
    if(promise) {
      if(err) {
        promise.reject(err);
      } else {
        promise.resolve(reportId);
      }
    }
  });
  return new Promise((resolve, reject) => {

    const id = uuid.v4();
    unAnsweredIpcCalls.set(id, { resolve, reject });
    ipcRenderer.send('collect-logs', id, toRedact);
    setTimeout(() => reapIpcCall(id), 1000);
  }).catch((e) => {
    const { err, stdout } = e;
    log.error('Failed collecting problem report', err);
    log.error('  stdout: ' + stdout);

    throw e;
  });
};

const sendProblemReport = (email: string, message: string, savedReport: string) => {
  const args = ['send',
    '--email', email,
    '--message', message,
    '--report', savedReport,
  ];

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