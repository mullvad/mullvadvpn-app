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

  IpcMainEventChannel.problemReport.handleViewLog((savedReportId) =>
    shell.openPath(getProblemReportPath(savedReportId)),
  );
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
