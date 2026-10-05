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

extension AccessMethodViewModel.TestingStatus {
    var viewStatus: MethodTestingStatusCellContentConfiguration.Status {
        switch self {
        case .initial:
            // The sheet is invisible in this state, the return value is not important.
            .testing
        case .inProgress:
            .testing
        case .failed:
            .unreachable
        case .succeeded:
            .reachable
        }
    }
}
