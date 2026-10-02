// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

struct RedactionPolicy {
    private let migratedVersion = "2026.4"

    func shouldRedact(content: String) -> Bool {
        guard
            let version = version(content: content),
            let current = Version(version.lowercased()),
            let target = Version(migratedVersion.lowercased())
        else {
            return true
        }

        return current < target
    }

    private func version(content: String) -> String? {
        // Example: MullvadVPN version 2026.1-dev11 -> 2026.1-dev11
        let pattern = #"version\s+(\d+\.\d+(?:-\w+)?)"#
        guard let header = content.components(separatedBy: .newlines).first else {
            return nil
        }

        guard let regex = try? Regex(pattern).ignoresCase(),
            let match = header.firstMatch(of: regex),
            let value = match.output.first?.value as? Substring
        else {
            return nil
        }

        return String(value)
            .components(separatedBy: .whitespaces)
            .dropFirst()
            .joined(separator: " ")
    }
}

private struct Version: Comparable {
    let major: Int
    let minor: Int

    init?(_ string: String) {
        let parts = string.split(separator: "-", maxSplits: 1)

        let numbers = parts[0].split(separator: ".")
        guard numbers.count == 2,
            let major = Int(numbers[0]),
            let minor = Int(numbers[1])
        else {
            return nil
        }

        self.major = major
        self.minor = minor
    }

    static func < (lhs: Version, rhs: Version) -> Bool {
        if lhs.major != rhs.major {
            return lhs.major < rhs.major
        }

        if lhs.minor != rhs.minor {
            return lhs.minor < rhs.minor
        }

        return false
    }
}
