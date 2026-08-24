const path = require('path');
const { execFileSync } = require('child_process');

const { WORKSPACE_PROJECT_ROOT } = require('./constants.cjs');

const SIGN_SCRIPT = path
  .resolve(WORKSPACE_PROJECT_ROOT, '../../..', 'scripts/sign-windows.sh')
  .replaceAll('\\', '/');

function signWindows(configuration) {
  const binary = configuration.path;
  execFileSync('bash', [SIGN_SCRIPT, binary], { stdio: 'inherit' });
  return Promise.resolve();
}

exports.signWindows = signWindows;
