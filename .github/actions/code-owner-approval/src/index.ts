import { retrieveApprovalState } from "./check";
import { info, error, setFailed } from "@actions/core";
import { getChangedFiles, setCommitStatus } from "./github";
import { loadTeamPatterns } from "./parse";

async function run() {
  // Put all logic in a try-catch, so we can set the commit status
  // to "error" if something fails.
  try {
    // Set status to pending at the start, clearing any previous state
    await setCommitStatus("pending", "Checking code owner approvals...");

    // Load team ownership mapping
    const codeOwnerships = loadTeamPatterns("code-owners.json");
    const changedFiles = await getChangedFiles();
    info(`Files changed in this PR: ${changedFiles}`);

    const approvals = await retrieveApprovalState(changedFiles, codeOwnerships);

    if (approvals.required.size == 0) {
      info("✅ No code owner for any changed file");
      await setCommitStatus(
        "success",
        "No changes require code owner approval",
      );

      return;
    }

    const mergeable =
      approvals.missing.size == 0 && approvals.missingApprovalCount == 0;

    const requiredApprovals = Array.from(approvals.required.values()).join(",");
    info(`👥 This PR needs approval from: ${requiredApprovals}`);

    const allCodeOwnersApproved = approvals.missing.size == 0;
    const approvalCountSatisfied = approvals.missingApprovalCount == 0;

    const requiredCodeOwners = Array.from(approvals.missing.values()).join(",");
    const codeOwnersMsg = allCodeOwnersApproved
      ? `Code owners approved!`
      : `Missing approvals from : ${requiredCodeOwners}.`;
    const approvalCountMsg = approvalCountSatisfied
      ? `Approval count: ✅`
      : `Approvals missing: ${approvals.missingApprovalCount}.`;

    const message = `${codeOwnersMsg} ${approvalCountMsg}`;
    if (mergeable) {
      info("✅ Sufficient code owners approved this change!");
      await setCommitStatus("success", `✅ ${message}`);

      return;
    }

    error(`❌ ${message}`);
    await setCommitStatus("failure", `❌ ${message}`);
  } catch (err) {
    await setCommitStatus("error", "Error checking code owner approvals");
    setFailed(`Checking approvals failed with ${err}`);
  }
}

run();
