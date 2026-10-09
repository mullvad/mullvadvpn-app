import { readFileSync } from "node:fs";

/** A single glob pattern. A leading `!` marks a negation (exclude) rule. */
export type FilePattern = string;

/**
 * The whole file: team name -> ordered list of glob patterns.
 * Order matters, because negations (`!...`) apply relative to earlier entries.
 */
export type TeamPatterns = Record<string, FilePattern[]>;

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

/** Type guard: narrows `unknown` (e.g. `JSON.parse` output) to `TeamPatterns`. */
function isTeamPatterns(value: unknown): value is TeamPatterns {
  return (
    isRecord(value) &&
    Object.values(value).every(
      (patterns) =>
        Array.isArray(patterns) && patterns.every((p) => typeof p === "string"),
    )
  );
}

/** Parse the raw JSON *string* you just read from disk. Throws if malformed. */
export function parseTeamPatterns(json: string): TeamPatterns {
  const data: unknown = JSON.parse(json);
  if (!isTeamPatterns(data)) {
    throw new TypeError(
      "Malformed team patterns: expected Record<string, string[]>",
    );
  }
  return data;
}

/** Read + parse straight from the file system. */
export function loadTeamPatterns(path: string): TeamPatterns {
  return parseTeamPatterns(readFileSync(path, "utf8"));
}
