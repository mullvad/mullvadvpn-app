//
//  ProxyConfigurationSectionIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ProxyConfigurationSectionIdentifier: Hashable {
    case `protocol`
    case proxyConfiguration

    var sectionName: String? {
        switch self {
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
