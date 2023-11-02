//
//  EditAccessMethodItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum EditAccessMethodItemIdentifier: Hashable {
    case name
    case useIfAvailable
    case proxyConfiguration
    case testMethod
    case testingStatus
    case deleteMethod

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .name:
            .textInput
        case .useIfAvailable:
            .toggle
        case .proxyConfiguration:
            .textWithDisclosure
        case .testMethod, .deleteMethod:
            .button
        case .testingStatus:
            .testingStatus
        }
    }

    /// Returns `true` if the cell background should be made transparent.
    var isClearBackground: Bool {
        switch self {
        case .testMethod, .testingStatus, .deleteMethod:
            return true
        case .name, .useIfAvailable, .proxyConfiguration:
            return false
        }
    }

    /// Whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case .name, .useIfAvailable, .testMethod, .testingStatus, .deleteMethod:
            false
        case .proxyConfiguration:
            true
        }
    }

    /// The text label for the corresponding cell.
    var text: String? {
        switch self {
        case .name:
            NSLocalizedString("NAME", tableName: "APIAccess", value: "Name", comment: "")
        case .useIfAvailable:
            NSLocalizedString("USE_IF_AVAILABLE", tableName: "APIAccess", value: "Use if available", comment: "")
        case .proxyConfiguration:
            NSLocalizedString("PROXY_CONFIGURATION", tableName: "APIAccess", value: "Proxy configuration", comment: "")
        case .testMethod:
            NSLocalizedString("TEST_METHOD", tableName: "APIAccess", value: "Test method", comment: "")
        case .testingStatus:
            nil
        case .deleteMethod:
            NSLocalizedString("DELETE_METHOD", tableName: "APIAccess", value: "Delete method", comment: "")
        }
    }
}
