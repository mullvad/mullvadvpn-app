import { readFileSync } from 'node:fs';

/** A single glob pattern. A leading `!` marks a negation (exclude) rule. */
export type FilePattern = string;

/**
 * The representation of one top-level entry in the JSON-file.
 *
 * This is one instance of TeamPatterns:
 * {
    "<team_name>": [ "**\/*.rust" ]
 * }
 */
export type TeamPatterns = Record<string, FilePattern[]>;

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === 'object' && v !== null && !Array.isArray(v);

/** Type guard: narrows `unknown` (e.g. `JSON.parse` output) to `TeamPatterns`. */
function isTeamPatterns(value: unknown): value is TeamPatterns {
  return (
    isRecord(value) &&
    Object.values(value).every(
      (patterns) => Array.isArray(patterns) && patterns.every((p) => typeof p === 'string'),
    )
  );
}

/** Parse the raw JSON *string*. Throws if malformed.
 * @see TeamPatterns
 * @throws TypeError if the pattern does not match Record<string, string[]>
 */
export function parseTeamPatterns(json: string): TeamPatterns {
  const data: unknown = JSON.parse(json);
  if (!isTeamPatterns(data)) {
    throw new TypeError('Malformed team patterns: expected Record<string, string[]>');
  }
  return data;
}

/** Read + parse straight from the file system. */
export function loadTeamPatterns(path: string): TeamPatterns {
  return parseTeamPatterns(readFileSync(path, 'utf8'));
}
