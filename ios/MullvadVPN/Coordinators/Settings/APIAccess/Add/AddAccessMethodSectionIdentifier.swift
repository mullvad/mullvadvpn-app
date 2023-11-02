//
//  AddAccessMethodSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum AddAccessMethodSectionIdentifier: Hashable {
    case name
    case `protocol`
    case proxyConfiguration

    /// The section name.
    var sectionName: String? {
        switch self {
        case .name:
            nil
        case .protocol:
            NSLocalizedString(
                "PROTOCOL_SECTION_TITLE",
                tableName: "APIAccess",
                value: "Protocol",
                comment: ""
            )
        case .proxyConfiguration:
            NSLocalizedString(
                "HOST_CONFIG_SECTION_TITLE",
                tableName: "APIAccess",
                value: "Host configuration",
                comment: ""
            )
        }
    }
}
