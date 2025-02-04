//
//  EditAccessMethodSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
            NSLocalizedString(
                "TEST_METHOD_FOOTER",
                tableName: "APIAccess",
                value: "Performs a connection test to a Mullvad API server via this access method.",
                comment: ""
            )

        case .enableMethod, .methodSettings, .cancelTest, .testingStatus, .deleteMethod:
            nil
        }
    }
}
