import log from 'electron-log';
import { shell, ipcRenderer } from 'electron';
import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import Support from '../components/Support';
import { resolveBin } from '../lib/proc';
import { execFile } from 'child_process';
import uuid from 'uuid';

const mapStateToProps = (state) => {
  return state;
};

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

  if (err) {
    promise.reject(err);
  } else if (promise) {
    promise.resolve(reportId);
  }
});

const mapDispatchToProps = (dispatch, _props) => {
  return {
    onClose: () => dispatch(push('/settings')),

    onCollectLog: () => {
      return new Promise((resolve, reject) => {

        const id = uuid.v4();
        unAnsweredIpcCalls.set(id, { resolve, reject });
        ipcRenderer.send('collect-logs', id);
        setTimeout(() => reapIpcCall(id), 1000);
      })
        .catch((e) => {
          const { err, stdout } = e;
          log.error('Failed collecting problem report', err);
          log.error('  stdout: ' + stdout);

          throw e;
        });
    },

    onViewLog: (path) => shell.openItem(path),

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
