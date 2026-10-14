import { error, info, setFailed } from '@actions/core';

import { retrieveApprovalState } from './check';
import { getChangedFiles, setCommitStatus } from './github';
import { loadTeamPatterns } from './parse';

async function run() {
  // Put all logic in a try-catch, so we can set the commit status
  // to "error" if something fails.
  try {
    // Set status to pending at the start, clearing any previous state
    await setCommitStatus('pending', 'Checking code owner approvals...');

    // Load team ownership mapping
    const codeOwnerships = loadTeamPatterns('code-owners.json');
    const changedFiles = await getChangedFiles();
    info(`Files changed in this PR: ${changedFiles}`);

    const approvals = await retrieveApprovalState(changedFiles, codeOwnerships);

    if (approvals.required.size == 0) {
      info('✅ No code owner for any changed file');
      await setCommitStatus('success', 'No changes require code owner approval');

      return;
    }

    const requiredApprovals = Array.from(approvals.required.values()).join(',');
    info(`👥 This PR needs approval from: ${requiredApprovals}`);

    if (approvals.missing.size == 0) {
      info('✅ All code owners approved this change!');
      await setCommitStatus('success', `All code owners have approved: ${requiredApprovals}`);

      return;
    }

    const missingApprovals = Array.from(approvals.missing.values()).join(',');

    error(`❌ Missing approvals from: ${missingApprovals}`);
    await setCommitStatus('failure', `Missing approvals from: ${missingApprovals}`);
  } catch (err) {
    await setCommitStatus('error', 'Error checking code owner approvals');
    setFailed(`Checking approvals failed with ${err}`);
  }
}

await run();
