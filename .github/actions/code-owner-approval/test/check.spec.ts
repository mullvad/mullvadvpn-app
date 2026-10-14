import path from 'node:path';

import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import * as github from '../src/github';

const { retrieveApprovalState } = await import('../src/check');

vi.mock(import('../src/github'), () => ({
  getPullRequestAuthor: vi.fn(),
  getApprovers: vi.fn().mockResolvedValue([]),
  setCommitStatus: vi.fn(),
  searchMatchingFiles: vi.fn().mockResolvedValue([]),
  isUserInTeam: vi.fn(),
}));

vi.mock(import('@actions/core'), () => ({
  info: vi.fn(),
  startGroup: vi.fn(),
  endGroup: vi.fn(),
}));

const mockedMatchingFiles = vi.mocked(github.searchMatchingFiles);
const mockedApprovers = vi.mocked(github.getApprovers);
const mockedIsInTeam = vi.mocked(github.isUserInTeam);
const mockedPullRequestAuthor = vi.mocked(github.getPullRequestAuthor);

describe('retrieveApprovalState', () => {
  beforeEach(() => {
    vi.stubEnv('GITHUB_WORKSPACE', '.');
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllEnvs();
  });

  test('returns required and missing approval sets', async () => {
    const result = await checkApproval([], {});

    expect(result).toHaveProperty('required');
    expect(result).toHaveProperty('missing');
  });

  describe('when no approvals are required', () => {
    test('no files changed and no owners configured', async () => {
      const result = await checkApproval([], {});

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });

    test('files changed but no owners configured', async () => {
      const result = await checkApproval(changed(['some']), {});

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });

    test('changed files are not covered by any owner pattern', async () => {
      const result = await checkApproval(changed(['a file']), owns({ team: ['another file'] }));

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });
  });

  describe('when approvals are required', () => {
    test('a non-owner approver does not satisfy the requirement', async () => {
      const result = await checkApproval(changed(['some']), owns({ team: ['some'] }), {
        approvers: approvedBy('non-owner'),
      });

      expect(result.required).toEqual(new Set(['team']));
      expect(result.missing).toEqual(new Set(['team']));
    });

    test('a code-owner approver satisfies the requirement', async () => {
      const result = await checkApproval(changed(['some']), owns({ team: ['some'] }), {
        approvers: approvedBy(DEFAULT_AUTHOR),
        authorIsMemberOfTeam: true,
      });

      expect(result.required).toEqual(new Set(['team']));
      expect(result.missing).toEqual(new Set());
    });

    describe('PR author', () => {
      test('counts toward approval when they are a team member', async () => {
        const result = await checkApproval(changed(['some']), owns({ team: ['some'] }), {
          approvers: approvedBy(DEFAULT_AUTHOR),
          authorIsMemberOfTeam: true,
        });

        expect(result.missing).toEqual(new Set());
      });

      test('does not count toward approval when they are not a team member', async () => {
        const result = await checkApproval(changed(['some']), owns({ team: ['some'] }), {
          approvers: approvedBy(DEFAULT_AUTHOR),
          authorIsMemberOfTeam: false,
        });

        expect(result.missing).toEqual(new Set(['team']));
      });

      test('is ignored when GitHub returns no author', async () => {
        const result = await checkApproval(changed(['some']), owns({ team: ['some'] }), {
          author: null,
          approvers: approvedBy('someone-else'),
        });

        expect(result.required).toEqual(new Set(['team']));
        expect(result.missing).toEqual(new Set(['team']));
      });
    });

    describe('multiple owners', () => {
      test('requires approval from every owner team', async () => {
        const result = await checkApproval(
          changed(['some']),
          owns({ team: ['some'], team2: ['some'], team3: ['some'] }),
        );

        expect(result.required).toEqual(new Set(['team', 'team2', 'team3']));
        expect(result.missing).toEqual(new Set(['team', 'team2', 'team3']));
      });

      test('each member-approver satisfies one owner requirement', async () => {
        const result = await checkApproval(
          changed(['some']),
          owns({ team: ['some'], team2: ['some'], team3: ['some'] }),
          {
            approvers: approvedBy(DEFAULT_AUTHOR),
            authorIsMemberOfTeam: (team) => team === 'team' || team === 'team2',
          },
        );

        expect(result.required).toEqual(new Set(['team', 'team2', 'team3']));
        expect(result.missing).toEqual(new Set(['team3']));
      });
    });
  });
});

const DEFAULT_AUTHOR = 'author';

function changed(files: string[]): string[] {
  return files;
}

function owns(mapping: Record<string, string[]>): Record<string, string[]> {
  return mapping;
}

function approvedBy(...users: string[]): string[] {
  return users;
}

/**
 * Default happy-path wiring. Individual tests override only what matters.
 */
async function checkApproval(
  changedFiles: string[],
  ownership: Record<string, string[]>,
  options: {
    author?: string | null;
    approvers?: string[];
    authorIsMemberOfTeam?: boolean | ((team: string) => boolean);
    globMatches?: string[];
  } = {},
) {
  const { author = DEFAULT_AUTHOR, approvers = [], authorIsMemberOfTeam = false } = options;

  mockedPullRequestAuthor.mockResolvedValue(author);
  mockedApprovers.mockResolvedValue(approvers);
  mockedIsInTeam.mockImplementation(({ team }) =>
    Promise.resolve(
      typeof authorIsMemberOfTeam === 'function'
        ? authorIsMemberOfTeam(team)
        : authorIsMemberOfTeam,
    ),
  );
  mockedMatchingFiles.mockImplementation((patterns) => {
    const res = changedFiles.filter((s) => patterns.some((p) => path.posix.matchesGlob(s, p)));
    return Promise.resolve(res);
  });

  return retrieveApprovalState(changedFiles, ownership);
}
