//
//  MethodSettingsSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum MethodSettingsSectionIdentifier: Hashable {
    case name
    case `protocol`
    case proxyConfiguration
    case validationError
    case testingStatus
    case cancelTest

    var sectionName: String? {
        switch self {
        case .name, .protocol, .validationError, .testingStatus, .cancelTest:
            nil
        case .proxyConfiguration:
            NSLocalizedString(
                "HOST_CONFIG_SECTION_TITLE",
                tableName: "APIAccess",
                value: "Server details",
                comment: ""
            )
        }
    }
}
