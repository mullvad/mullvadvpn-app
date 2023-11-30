//
//  AccessMethodActionSheetContentConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Sheet content view configuration.
struct AccessMethodActionSheetContentConfiguration: Equatable {
    /// The status of access method testing.
    enum Status: Equatable {
        /// API Is reachable.
        case reachable

        /// API is unreachable.
        case unreachable

        /// API testing is in progress.
        case testing
    }

    /// The status of testing.
    var status: Status = .reachable

    /// Detail text displayed below the status when set.
    var detailText: String?
}

extension AccessMethodActionSheetContentConfiguration.Status {
    /// The text label descirbing the status of testing and suitable for user presentation.
    var text: String {
        switch self {
        case .unreachable:
            NSLocalizedString("API_UNREACHABLE", tableName: "APIAccess", value: "API unreachable", comment: "")
        case .reachable:
            NSLocalizedString("API_REACHABLE", tableName: "APIAccess", value: "API reachable", comment: "")
        case .testing:
            NSLocalizedString("API_TESTING_INPROGRESS", tableName: "APIAccess", value: "Testing...", comment: "")
        }
    }

    /// The color of a circular status indicator view.
    var statusColor: UIColor? {
        switch self {
        case .unreachable:
            .dangerColor
        case .reachable:
            .successColor
        case .testing:
            nil
        }
    }
}
