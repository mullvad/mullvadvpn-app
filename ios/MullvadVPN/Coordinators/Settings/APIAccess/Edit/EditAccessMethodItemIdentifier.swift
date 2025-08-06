//
//  EditAccessMethodItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
            NSLocalizedString("ENABLE_METHOD", value: "Enable method", comment: "")
        case .methodSettings:
            NSLocalizedString("METHOD_SETTINGS", value: "Method settings", comment: "")
        case .testMethod:
            NSLocalizedString("TEST_METHOD", value: "Test method", comment: "")
        case .cancelTest:
            NSLocalizedString("CANCEL_TITLE_BUTTON", comment: "")
        case .testingStatus:
            nil
        case .deleteMethod:
            NSLocalizedString("DELETE_METHOD", value: "Delete method", comment: "")
        }
    }
}
