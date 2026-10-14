import { endGroup, info, startGroup } from '@actions/core';

import { getApprovers, getPullRequestAuthor, isUserInTeam, searchMatchingFiles } from './github';
import { TeamPatterns } from './parse';

async function determineRequiredReviewers(
  changedFiles: string[],
  ownerShipRules: TeamPatterns,
): Promise<Set<string>> {
  // The set of teams owning code changed in this PR
  const affectedTeams = new Set<string>();

  for (const [team, patterns] of Object.entries(ownerShipRules)) {
    startGroup(`Checking team ${team} ownership`);
    info(`[Team: ${team}, Ownership patterns: ${[...patterns]}`);

    const teamOwnedFiles = await searchMatchingFiles(patterns);

    for (const changedFile of changedFiles) {
      if (teamOwnedFiles.includes(changedFile)) {
        affectedTeams.add(team);
        info(`File ${changedFile} is owned by ${team}`);
      }
    }
    endGroup();
  }

  return affectedTeams;
}

async function determineMissingReviewers(affectedTeams: Set<string>) {
  // Set of teams that have approved this PR
  const approvedTeams = new Set<string>();

  // Get array of github usernames that have approved the PR
  const approvers = await getApprovers();

  // PR author automatically counts as an approver. A code owner changing
  // their own code does not need extra code owner approval. They still
  // need the code reviewed, but that's not part of code ownership approval.
  const prAuthor = await getPullRequestAuthor();
  if (prAuthor && !approvers.includes(prAuthor)) {
    approvers.push(prAuthor);
  }
  info(`👍 PR approved by the following accounts: ${approvers.join(', ')}`);

  for (const approver of approvers) {
    for (const team of affectedTeams) {
      const countsAsApproval = await isUserInTeam({ user: approver, team });
      if (countsAsApproval) {
        info(`${approver} is member of team '${team}' - approval counted`);
        approvedTeams.add(team);
      } else {
        info(`${approver} is not member of team '${team}'`);
      }
    }
  }

  info(`👍 Teams that have approved this PR: ${[...approvedTeams].join(', ')}`);

  const missingApprovals = [...affectedTeams].filter((t) => !approvedTeams.has(t));

  return new Set(missingApprovals);
}

type Approvers = {
  required: Set<string>;
  missing: Set<string>;
};

export async function retrieveApprovalState(
  changedFiles: string[],
  codeOwnerships: TeamPatterns,
): Promise<Approvers> {
  const affectedTeams = await determineRequiredReviewers(changedFiles, codeOwnerships);

  if (affectedTeams.size === 0) {
    return { required: new Set(), missing: new Set() };
  }

  const missingApprovals = await determineMissingReviewers(affectedTeams);
  return { required: affectedTeams, missing: missingApprovals };
}
