// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public enum MigratedVersion: Int, Sendable, Equatable {
    case v1 = 1
    case v2 = 2

    var nextVersion: MigratedVersion {
        switch self {
        case .v1:
            .v2
        case .v2:
            .v2
        }
    }

    public static let current: MigratedVersion = .v2

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.rawValue == rhs.rawValue
    }
}
