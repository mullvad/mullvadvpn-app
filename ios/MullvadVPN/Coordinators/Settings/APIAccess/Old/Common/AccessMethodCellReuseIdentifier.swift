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

/// Cell reuse identifier used by table view controllers implementing various parts of API access management.
enum AccessMethodCellReuseIdentifier: String, CaseIterable, CellIdentifierProtocol {
    /// Cells with static text and disclosure view.
    case textWithDisclosure

    /// Cells with a label and text field.
    case textInput

    /// Cells with a label and switch control.
    case toggle

    /// Cells that contain a button.
    case button

    /// Cells that contain a number of validation errors.
    case validationError

    /// Cells that contain the status of API method testing.
    case testingStatus

    var cellClass: AnyClass {
        BasicCell.self
    }
}
