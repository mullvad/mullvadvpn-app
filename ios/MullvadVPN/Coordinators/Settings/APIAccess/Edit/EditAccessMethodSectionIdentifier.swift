//
//  EditAccessMethodSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum EditAccessMethodSectionIdentifier: Hashable {
    case name
    case testMethod
    case useIfAvailable
    case proxyConfiguration
    case deleteMethod

    /// The section footer text.
    var sectionFooter: String? {
        switch self {
        case .name, .deleteMethod:
            nil

        case .testMethod:
            NSLocalizedString(
                "TEST_METHOD_FOOTER",
                tableName: "APIAccess",
                value: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt",
                comment: ""
            )

        case .useIfAvailable:
            NSLocalizedString(
                "USE_IF_AVAILABLE_FOOTER",
                tableName: "APIAccess",
                value: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt",
                comment: ""
            )

        case .proxyConfiguration:
            NSLocalizedString(
                "PROXY_CONFIGURATION_FOOTER",
                tableName: "APIAccess",
                value: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt",
                comment: ""
            )
        }
    }
}
