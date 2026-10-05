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

enum EditAccessMethodSectionIdentifier: Hashable {
    case enableMethod
    case methodSettings
    case testMethod
    case cancelTest
    case testingStatus
    case deleteMethod

    /// The section footer text.
    var sectionFooter: String? {
        switch self {
        case .testMethod:
            NSLocalizedString("Performs a connection test to a Mullvad API server via this access method.", comment: "")
        case .enableMethod:
            NSLocalizedString("At least one method needs to be enabled.", comment: "")
        case .methodSettings, .cancelTest, .testingStatus, .deleteMethod:
            nil
        }
    }
}
