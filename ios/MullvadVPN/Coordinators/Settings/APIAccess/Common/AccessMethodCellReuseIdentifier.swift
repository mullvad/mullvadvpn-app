//
//  AccessMethodCellReuseIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
