import { parseTeamPatterns } from "../src/parse";

describe("parseTeamPatterns", () => {
  it("accepts an empty object as a sane default", () => {
    expect(parseTeamPatterns("{}")).toEqual({});
  });

  it("parses a raw JSON string and retains the glob-patterns", () => {
    const json = JSON.stringify({
      "appteam-desktop": ["desktop/**", "mullvad-*/**"],
      "appteam-ios": ["ios/**", "ci/ios/**"],
    });
    const result = parseTeamPatterns(json);

    expect(Object.keys(result)).toEqual(["appteam-desktop", "appteam-ios"]);
    expect(result["appteam-desktop"]).toEqual(["desktop/**", "mullvad-*/**"]);
    expect(result["appteam-ios"]).toEqual(["ios/**", "ci/ios/**"]);
  });

  it("throws if the ownership line is not an array", () => {
    const json = JSON.stringify({ "bad-team": "*" });
    expect(() => parseTeamPatterns(json)).toThrow(
      "Malformed team patterns: expected Record<string, string[]>",
    );
  });
});
