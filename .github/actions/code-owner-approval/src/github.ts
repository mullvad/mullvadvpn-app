import { getOctokit, context } from "@actions/github";
import { debug, getInput } from "@actions/core";
import * as glob from "@actions/glob";
import * as path from "path";

// These are the accepted states according to
// https://docs.github.com/en/rest/commits/statuses?apiVersion=2026-03-10
type CommitStatusState = "error" | "failure" | "pending" | "success";

// Retrieve the `Review` type so we have a type-safe way of referring to it.
type Octokit = ReturnType<typeof getOctokit>;
type Review = Awaited<
  ReturnType<Octokit["rest"]["pulls"]["listReviews"]>
>["data"][number];

let octokit: ReturnType<typeof getOctokit> | undefined;

function getOctokitClient() {
  if (!octokit) {
    const token = getInput("github-token", { required: true });
    octokit = getOctokit(token);
  }
  return octokit;
}

// Helper function to set a custom commit status. This is not the status
// of this workflow run, but rather a custom status.
// This is required since the `pull_request` and `pull_request_review` events
// trigger independently and create separate statuses, that can be incompatible.
// Instead we report both of these into a custom status, that is set as required
// in the branch protection rules.
export async function setCommitStatus(
  state: CommitStatusState,
  description: string,
) {
  if (context.payload.pull_request === undefined) {
    throw new Error("Pull request is undefined, aborting");
  }

  const { owner, repo } = context.repo;
  await getOctokitClient().rest.repos.createCommitStatus({
    owner: owner,
    repo: repo,
    sha: context.payload.pull_request.head.sha,
    state: state,
    context: "Code Owner Approval",
    description: description,
  });
}

// Returns an array of file paths changed in the PR
export async function getChangedFiles() {
  const octokit = getOctokitClient();
  const { number: issueNumber } = context.issue;
  const { owner, repo } = context.repo;
  const changedFiles = await octokit.paginate(octokit.rest.pulls.listFiles, {
    owner: owner,
    repo: repo,
    pull_number: issueNumber,
  });

  return changedFiles.map((file) => file.filename);
}

// Returns a list of usernames who approved the PR (based on their latest review)
export async function getApprovers() {
  const octokit = getOctokitClient();
  const { owner, repo } = context.repo;
  const { number: issueNumber } = context.issue;
  const reviews = await octokit.paginate(octokit.rest.pulls.listReviews, {
    owner: owner,
    repo: repo,
    pull_number: issueNumber,
  });

  const latestReviews = new Map<number, Review>();

  for (const review of reviews) {
    if (!review.user) {
      continue;
    }

    const currentLatest = latestReviews.get(review.user.id);

    // Keep the most recent review (higher ID = more recent)
    if (!currentLatest || review.id > currentLatest.id) {
      latestReviews.set(review.user.id, review);
    }
  }

  // Filter to only approved reviews
  const approvers = new Array<string>();
  for (const [_, review] of latestReviews) {
    if (review.state === "APPROVED" && review.user) {
      approvers.push(review.user.login);
    }
  }

  return approvers;
}

export async function getTeamMembers(team: string): Promise<string[]> {
  const octokit = getOctokitClient();
  const { owner } = context.repo;

  const members = await octokit.paginate(octokit.rest.teams.listMembersInOrg, {
    org: owner,
    team_slug: team,
  });

  return members.map((m) => m.login);
}

export async function isFork() {
  const client = getOctokitClient();
  const prNumber = context.payload.pull_request?.number;
  if (!prNumber) {
    return false;
  }

  const { owner, repo } = context.repo;
  const res = await client.rest.pulls.get({
    owner,
    repo,
    pull_number: prNumber,
  });

  return res.data.head.repo.fork;
}

export function getPullRequestAuthor() {
  return context.payload.pull_request?.user?.login;
}

export async function isUserInTeam({
  user,
  team,
}: {
  user: string;
  team: string;
}) {
  try {
    const octokit = getOctokitClient();
    const { owner } = context.repo;
    const response = await octokit.rest.teams.getMembershipForUserInOrg({
      org: owner,
      team_slug: team,
      username: user,
    });
    return response.data.state == "active";
  } catch (e) {
    debug(`Retrievinng team membership for ${user} failed with ${e}`);
    return false;
  }
}

export async function searchMatchingFiles(patterns: string[]) {
  // List all files in the repository matching this owner's patterns
  const globber = await glob.create(patterns.join("\n"));
  const matches = await globber.glob();

  const workspace = process.env.GITHUB_WORKSPACE;
  if (workspace === undefined) {
    throw new Error("GITHUB_WORKSPACE not set");
  }

  // Convert absolute paths to relative paths
  return matches.map((match) => path.relative(workspace, match));
}
