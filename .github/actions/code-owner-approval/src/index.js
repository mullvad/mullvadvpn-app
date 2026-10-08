import { checkApprovals, setCommitStatus } from './check.js';
import { setFailed } from '@actions/core';

async function run() {
  // Put all logic in a try-catch, so we can set the commit status
  // to "error" if something fails.
  try {
    await checkApprovals();
  } catch (err) {
    await setCommitStatus('error', 'Error checking code owner approvals');
    setFailed(`Checking approvals failed with ${err}`);
  }
}

run();
