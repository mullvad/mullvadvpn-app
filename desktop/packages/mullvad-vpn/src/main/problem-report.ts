import { execFile } from 'child_process';
import { randomUUID } from 'crypto';
import { app, shell } from 'electron';
import * as path from 'path';

import log from '../shared/logging';
import { IpcMainEventChannel } from './ipc-event-channel';
import { resolveBin } from './proc';

export function registerIpcListeners() {
  IpcMainEventChannel.problemReport.handleCollectLogs(collectLogs);

  IpcMainEventChannel.problemReport.handleSendReport(({ email, message, savedReportId }) => {
    return send(email, message, savedReportId);
  });

  IpcMainEventChannel.problemReport.handleViewLog((savedReportId) => {
    const problemReportPath = getProblemReportPath(savedReportId);
    if (process.platform === 'linux') {
      // As of this upstream PR[1] the underlying C implementation for
      // shell.openPath no longer waits for the process to exit, which
      // means that the callback in the C code will never be called.
      //
      // That callback is what eventually causes the promise returned
      // by shell.openPath to resolve, and as it is never being called,
      // the promise will never be resolved.
      //
      // Because of that, we just invoke shell.openPath and return a
      // promise resolved with an empty string, the same signature as
      // returned from shell.openPath.
      //
      // [1] https://github.com/electron/electron/pull/48079
      void shell.openPath(problemReportPath);
      return Promise.resolve('');
    }

    return shell.openPath(problemReportPath);
  });
}

function collectLogs(toRedact?: string): Promise<string> {
  const id = randomUUID();
  const reportPath = getProblemReportPath(id);
  const executable = resolveBin('mullvad-problem-report');
  const args = ['collect', '--output', reportPath];
  if (toRedact) {
    args.push('--redact', toRedact);
  }

  return new Promise((resolve, reject) => {
    execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
      if (error) {
        log.error(
          `Failed to collect a problem report.
            Stdout: ${stdout.toString()}
            Stderr: ${stderr.toString()}`,
        );
        reject(error.message);
      } else {
        log.verbose(`Problem report was written to ${reportPath}`);
        resolve(id);
      }
    });
  });
}

function send(email: string, message: string, savedReportId: string): Promise<void> {
  const executable = resolveBin('mullvad-problem-report');
  const reportPath = getProblemReportPath(savedReportId);
  const args = ['send', '--email', email, '--message', message, '--report', reportPath];

  return new Promise((resolve, reject) => {
    execFile(executable, args, { windowsHide: true }, (error, stdout, stderr) => {
      if (error) {
        log.error(
          `Failed to send a problem report.
          Stdout: ${stdout.toString()}
          Stderr: ${stderr.toString()}`,
        );
        reject(error.message);
      } else {
        log.info('Problem report was sent.');
        resolve();
      }
    });
  });
}

function getProblemReportPath(id: string): string {
  return path.join(app.getPath('temp'), `${id}.log`);
}
