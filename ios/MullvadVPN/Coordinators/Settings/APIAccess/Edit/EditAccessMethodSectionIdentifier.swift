//
//  EditAccessMethodSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
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
        case .enableMethod:
            NSLocalizedString(
                "ENABLE_METHOD_FOOTER",
                tableName: "APIAccess",
                value: "When enabled, the app can try to communicate with a Mullvad API server using this method.",
                comment: ""
            )

        case .testMethod:
            NSLocalizedString(
                "TEST_METHOD_FOOTER",
                tableName: "APIAccess",
                value: "Performs a connection test to a Mullvad API server via this access method.",
                comment: ""
            )

        case .methodSettings, .cancelTest, .testingStatus, .deleteMethod:
            nil
        }
    }
}
