import { describe, vi, beforeEach, afterEach, expect, test } from "vitest";
import path from "node:path";
import * as github from "../src/github";

const { retrieveApprovalState } = await import("../src/check");

vi.mock(import("../src/github"), () => ({
  getPullRequestAuthor: vi.fn(),
  getApprovers: vi.fn().mockResolvedValue([]),
  setCommitStatus: vi.fn(),
  searchMatchingFiles: vi.fn().mockResolvedValue([]),
  isUserInTeam: vi.fn(),
  isFork: vi.fn(),
  getTeamMembers: vi.fn(),
}));

vi.mock(import("@actions/core"), async () => ({
  info: vi.fn(),
  startGroup: vi.fn(),
  endGroup: vi.fn(),
}));

const mockedMatchingFiles = vi.mocked(github.searchMatchingFiles);
const mockedApprovers = vi.mocked(github.getApprovers);
const mockedIsInTeam = vi.mocked(github.isUserInTeam);
const mockedTeamMembers = vi.mocked(github.getTeamMembers);
const mockedPullRequestAuthor = vi.mocked(github.getPullRequestAuthor);

describe("retrieveApprovalState", () => {
  beforeEach(() => {
    vi.stubEnv("GITHUB_WORKSPACE", ".");
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllEnvs();
  });

  test("returns required and missing approval sets", async () => {
    const result = await checkApproval([], {});

    expect(result).toHaveProperty("required");
    expect(result).toHaveProperty("missing");
  });

  describe("when no approvals are required", () => {
    test("no files changed and no owners configured", async () => {
      const result = await checkApproval([], {});

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });

    test("files changed but no owners configured", async () => {
      const result = await checkApproval(changed(["some"]), {});

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });

    test("changed files are not covered by any owner pattern", async () => {
      const result = await checkApproval(
        changed(["a file"]),
        owns({ team: ["another file"] }),
      );

      expect(result.required).toHaveLength(0);
      expect(result.missing).toHaveLength(0);
    });
  });

  describe("when approvals are required", () => {
    test("a non-owner approver does not satisfy the requirement", async () => {
      const result = await checkApproval(
        changed(["some"]),
        owns({ team: ["some"] }),
        { approvers: approvedBy("non-owner") },
      );

      expect(result.required).toEqual(new Set(["team"]));
      expect(result.missing).toEqual(new Set(["team"]));
    });

    test("a code-owner approver satisfies the requirement", async () => {
      const result = await checkApproval(
        changed(["some"]),
        owns({ team: ["some"] }),
        {
          approvers: approvedBy(DEFAULT_AUTHOR),
          approverIsMemberOfTeam: true,
        },
      );

      expect(result.required).toEqual(new Set(["team"]));
      expect(result.missing).toEqual(new Set());
    });

    test("multiple patterns for the same team match a changed file", async () => {
      const result = await checkApproval(
        changed(["index.js"]),
        owns({ team: ["*.ts", "*.js"] }),
        {
          author: "x",
          approvers: ["member"],
          teamMembers: ["member"],
          approverIsMemberOfTeam: true,
        },
      );

      expect(result.required).toEqual(new Set(["team"]));
      expect(result.missing).toEqual(new Set());
    });

    describe("PR author", () => {
      test("counts toward approval when they are a team member", async () => {
        const result = await checkApproval(
          changed(["some"]),
          owns({ team: ["some"] }),
          {
            approvers: [],
            approverIsMemberOfTeam: true,
          },
        );

        expect(result.missing).toEqual(new Set());
      });

      test("does not count toward approval when they are not a team member", async () => {
        const result = await checkApproval(
          changed(["some"]),
          owns({ team: ["some"] }),
          {
            approvers: [],
            approverIsMemberOfTeam: false,
          },
        );

        expect(result.missing).toEqual(new Set(["team"]));
      });

      test("is ignored when GitHub returns no author", async () => {
        const result = await checkApproval(
          changed(["some"]),
          owns({ team: ["some"] }),
          {
            author: null,
            approvers: approvedBy("someone-else"),
          },
        );

        expect(result.required).toEqual(new Set(["team"]));
        expect(result.missing).toEqual(new Set(["team"]));
      });
    });

    describe("multiple owners", () => {
      test("requires approval from every owner team", async () => {
        const result = await checkApproval(
          changed(["some"]),
          owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
        );

        expect(result.required).toEqual(new Set(["team", "team2", "team3"]));
        expect(result.missing).toEqual(new Set(["team", "team2", "team3"]));
      });

      test("each member-approver satisfies one owner requirement", async () => {
        const result = await checkApproval(
          changed(["some"]),
          owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
          {
            approvers: approvedBy(DEFAULT_AUTHOR, "member2"),
            teamMembers: [DEFAULT_AUTHOR, "member2"],
            approverIsMemberOfTeam: (user, team) =>
              (user == DEFAULT_AUTHOR && team == "team") ||
              (user == "member2" && team == "team2"),
          },
        );

        expect(result.required).toEqual(new Set(["team", "team2", "team3"]));
        expect(result.missing).toEqual(new Set(["team3"]));
      });

      test("a single approver can satisfy multiple affected teams", async () => {
        const result = await checkApproval(
          changed(["a", "b"]),
          owns({ team: ["a"], team2: ["b"] }),
          {
            author: "x",
            approvers: ["multi-member"],
            teamMembers: ["multi-member"],
            approverIsMemberOfTeam: (user, team) =>
              user === "multi-member" && (team === "team" || team === "team2"),
          },
        );

        expect(result.required).toEqual(new Set(["team", "team2"]));
        expect(result.missing).toEqual(new Set());
      });
    });

    describe("a 3rd party PR", () => {
      describe("review count", () => {
        test("requires 2 approvals on code-owner-related changes by default", async () => {
          const result = await checkApproval(
            changed(["some"]),
            owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
            {
              teamMembers: [],
            },
          );

          expect(result.missingApprovalCount).toEqual(2);
          expect(result.missing.size).toEqual(3);
        });

        test("requires 1 approval upon one approval", async () => {
          const result = await checkApproval(
            changed(["some"]),
            owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
            {
              author: "x",
              approvers: ["a"],
              teamMembers: ["a", "b"],
            },
          );

          expect(result.missingApprovalCount).toEqual(1);
        });

        test("requires 0 after having received 2 approvals from team members", async () => {
          const result = await checkApproval(
            changed(["some"]),
            owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
            {
              author: "x",
              approvers: ["a", "c"],
              teamMembers: ["a", "b", "c"],
            },
          );

          expect(result.missingApprovalCount).toEqual(0);
        });
      });

      describe("code owners", () => {
        test("are required even when a fellow appteam-member approves", async () => {
          const result = await checkApproval(
            changed(["some"]),
            owns({ team: ["some"], team2: ["some"], team3: ["some"] }),
            {
              approverIsMemberOfTeam: false,
              approvers: ["a"],
              author: "x",
              teamMembers: ["a", "b"],
            },
          );

          expect(result.missingApprovalCount).toEqual(1);
          expect(result.required).toHaveLength(3);
          expect(result.missing).toEqual(new Set(["team", "team2", "team3"]));
        });
      });
    });
  });
});

const DEFAULT_AUTHOR = "author";

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
    approverIsMemberOfTeam?:
      boolean | ((user: string, team: string) => boolean);
    globMatches?: string[];
    teamMembers?: string[];
  } = {},
) {
  const {
    author = DEFAULT_AUTHOR,
    approvers = [],
    approverIsMemberOfTeam = false,
    teamMembers = [DEFAULT_AUTHOR],
  } = options;

  mockedPullRequestAuthor.mockReturnValue(author);
  mockedApprovers.mockResolvedValue(approvers);
  mockedTeamMembers.mockResolvedValue(teamMembers);

  mockedIsInTeam.mockImplementation(({ user, team }) =>
    Promise.resolve(
      typeof approverIsMemberOfTeam === "function"
        ? approverIsMemberOfTeam(user, team)
        : approverIsMemberOfTeam,
    ),
  );
  mockedMatchingFiles.mockImplementation((patterns) => {
    const res = changedFiles.filter((s) =>
      patterns.some((p) => path.posix.matchesGlob(s, p)),
    );
    return Promise.resolve(res);
  });

  return retrieveApprovalState(changedFiles, ownership);
}
