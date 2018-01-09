// @flow

import { log, openItem } from '../lib/platform';
import { shell, ipcRenderer } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Support from '../components/Support';
import { resolveBin } from '../lib/proc';
import { execFile } from 'child_process';
import uuid from 'uuid';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => state;

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

const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);

  return {
    onClose: () => pushHistory('/settings'),

    onCollectLog: (toRedact) => {
      return new Promise((resolve, reject) => {

        const id = uuid.v4();
        unAnsweredIpcCalls.set(id, { resolve, reject });
        ipcRenderer.send('collect-logs', id, toRedact);
        setTimeout(() => reapIpcCall(id), 1000);
      })
        .catch((e) => {
          const { err, stdout } = e;
          log.error('Failed collecting problem report', err);
          log.error('  stdout: ' + stdout);

          throw e;
        });
    },

    onViewLog: (path) => openItem(path),

    onSend: (email, message, savedReport) => {

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
      })
        .catch((e) => {
          const { err, stdout } = e;
          log.error('Failed sending problem report', err);
          log.error('  stdout: ' + stdout);

          throw e;
        });
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Support);
