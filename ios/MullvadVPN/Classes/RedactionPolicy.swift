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
        let pattern = #"version\s+(\d+\.\d+(?:-\w+)?)"#
        guard let header = content.components(separatedBy: .newlines).first?.lowercased() else {
            return nil
        }

        guard let regex = try? Regex(pattern),
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
    let stage: Stage
    let stageNumber: Int

    enum Stage: Int, Comparable {
        case dev = 0
        case beta = 1
        case release = 2

        static func < (lhs: Stage, rhs: Stage) -> Bool {
            lhs.rawValue < rhs.rawValue
        }
    }

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

        guard parts.count == 2 else {
            self.stage = .release
            self.stageNumber = 0
            return
        }

        let suffix = parts[1]

        if suffix.hasPrefix("dev") {
            self.stage = .dev
            self.stageNumber = Int(suffix.dropFirst(3)) ?? 0
        } else if suffix.hasPrefix("beta") {
            self.stage = .beta
            self.stageNumber = Int(suffix.dropFirst(4)) ?? 0
        } else {
            return nil
        }
    }

    static func < (lhs: Version, rhs: Version) -> Bool {
        (lhs.major, lhs.minor, lhs.stage, lhs.stageNumber) < (rhs.major, rhs.minor, rhs.stage, rhs.stageNumber)
    }
}
