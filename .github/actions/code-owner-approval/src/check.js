import * as github from "@actions/github";
import * as fs from "fs";
import * as path from "path";
import * as glob from "@actions/glob";
import * as core from "@actions/core";
import {
  debug,
  info,
  error,
  startGroup,
  endGroup,
  setFailed,
} from "@actions/core";

const token = core.getInput("github-token", { required: true });
const octokit = github.getOctokit(token);
const { context } = github;

// Helper function to set a custom commit status. This is not the status
// of this workflow run, but rather a custom status.
// This is required since the `pull_request` and `pull_request_review` events
// trigger independently and create separate statuses, that can be incompatible.
// Instead we report both of these into a custom status, that is set as required
// in the branch protection rules.
async function setCommitStatus(state, description) {
  await octokit.rest.repos.createCommitStatus({
    owner: context.repo.owner,
    repo: context.repo.repo,
    sha: context.payload.pull_request.head.sha,
    state: state,
    context: "Code Owner Approval",
    description: description,
  });
}

// Returns an array of file paths changed in the PR
async function getChangedFiles() {
  const changedFiles = await octokit.paginate(octokit.rest.pulls.listFiles, {
    owner: context.repo.owner,
    repo: context.repo.repo,
    pull_number: context.issue.number,
  });

  return changedFiles.map((file) => file.filename);
}

// Returns a list of usernames who approved the PR (based on their latest review)
async function getApprovers() {
  const reviews = await octokit.paginate(octokit.rest.pulls.listReviews, {
    owner: context.repo.owner,
    repo: context.repo.repo,
    pull_number: context.issue.number,
  });

  const latestReviews = new Map();

  for (const review of reviews) {
    const currentLatest = latestReviews.get(review.user.id);

    // Keep the most recent review (higher ID = more recent)
    if (!currentLatest || review.id > currentLatest.id) {
      latestReviews.set(review.user.id, review);
    }
  }

  // Filter to only approved reviews
  const approvers = [];
  for (const [userId, review] of latestReviews) {
    if (review.state === "APPROVED") {
      approvers.push(review.user.login);
    }
  }

  return approvers;
}

async function checkApprovals() {
  // Set status to pending at the start, clearing any previous state
  await setCommitStatus("pending", "Checking code owner approvals...");

  const changedFiles = await getChangedFiles();
  debug("Files changed in this PR:", changedFiles);

  // Load team ownership mapping
  const codeOwnerships = JSON.parse(fs.readFileSync("code-owners.json"));

  // The set of teams owning code changed in this PR
  const affectedTeams = new Set();

  for (const [team, patterns] of Object.entries(codeOwnerships)) {
    startGroup(`Checking team ${team} ownership`);
    debug(`[Team: ${team}, Ownership patterns:`, [...patterns]);

    // List all files in the repository matching this owner's patterns
    const globber = await glob.create(patterns.join("\n"));
    const matches = await globber.glob();

    // Convert absolute paths to relative paths
    const ownedFiles = matches.map((match) =>
      path.relative(process.env.GITHUB_WORKSPACE, match),
    );

    for (const changedFile of changedFiles) {
      if (ownedFiles.includes(changedFile)) {
        affectedTeams.add(team);
        debug(`File ${changedFile} is owned by ${team}`);
      }
    }
    endGroup();
  }

  if (affectedTeams.size === 0) {
    info("✅ No code owner for any changed file");
    await setCommitStatus("success", "No changes require code owner approval");
    return;
  }

  info(`👥 This PR needs approval from: ${[...affectedTeams].join(", ")}`);

  // Set of teams that have approved this PR
  const approvedTeams = new Set();

  // Get array of github usernames that have approved the PR
  const approvers = await getApprovers();

  // PR author automatically counts as an approver. A code owner changing
  // their own code does not need extra code owner approval. They still
  // need the code reviewed, but that's not part of code ownership approval.
  const prAuthor = context.payload.pull_request.user.login;
  if (!approvers.includes(prAuthor)) {
    approvers.push(prAuthor);
  }
  info(`👍 PR approved by the following accounts: ${approvers.join(", ")}`);

  for (const approver of approvers) {
    for (const team of affectedTeams) {
      try {
        await octokit.rest.teams.getMembershipForUserInOrg({
          org: context.repo.owner,
          team_slug: team,
          username: approver,
        });
        approvedTeams.add(team);
        debug(`${approver} is member of team '${team}' - approval counted`);
      } catch (e) {
        debug(`${approver} is not member of team '${team}' (${e})`);
      }
    }
  }

  info("👍 Teams that have approved this PR:", [...approvedTeams].join(", "));

  const missingApprovals = [...affectedTeams].filter(
    (t) => !approvedTeams.has(t),
  );

  if (missingApprovals.length > 0) {
    error(`❌ Missing approvals from: ${missingApprovals.join(", ")}`);
    await setCommitStatus(
      "failure",
      `Missing approvals from: ${missingApprovals.join(", ")}`,
    );
  } else {
    info("✅ All code owners approved this change!");
    await setCommitStatus(
      "success",
      `All code owners have approved: ${[...affectedTeams].join(", ")}`,
    );
  }
}

export { checkApprovals, setCommitStatus };
