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

enum EditAccessMethodItemIdentifier: Hashable {
    case enableMethod
    case methodSettings
    case testMethod
    case testingStatus
    case cancelTest
    case deleteMethod

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .enableMethod:
            .toggle
        case .methodSettings:
            .textWithDisclosure
        case .testMethod, .cancelTest, .deleteMethod:
            .button
        case .testingStatus:
            .testingStatus
        }
    }

    /// Returns `true` if the cell background should be made transparent.
    var isClearBackground: Bool {
        switch self {
        case .testMethod, .cancelTest, .testingStatus, .deleteMethod:
            return true
        case .enableMethod, .methodSettings:
            return false
        }
    }

    /// Whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case .enableMethod, .testMethod, .cancelTest, .testingStatus, .deleteMethod:
            false
        case .methodSettings:
            true
        }
    }

    /// The text label for the corresponding cell.
    var text: String? {
        switch self {
        case .enableMethod:
            NSLocalizedString("Enable method", comment: "")
        case .methodSettings:
            NSLocalizedString("Edit method", comment: "")
        case .testMethod:
            NSLocalizedString("Test method", comment: "")
        case .cancelTest:
            NSLocalizedString("Cancel", comment: "")
        case .testingStatus:
            nil
        case .deleteMethod:
            NSLocalizedString("Delete method", comment: "")
        }
    }
}
