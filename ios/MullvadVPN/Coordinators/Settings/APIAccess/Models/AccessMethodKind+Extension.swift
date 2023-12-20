//
//  AccessMethodKind+Extension.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-12-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

extension AccessMethodKind {
    /// Returns localized description describing the access method.
    var localizedDescription: String {
        switch self {
        case .direct:
            NSLocalizedString("DIRECT", tableName: "APIAccess", value: "Direct", comment: "")
        case .bridges:
            NSLocalizedString("BRIDGES", tableName: "APIAccess", value: "Bridges", comment: "")
        case .shadowsocks:
            NSLocalizedString("SHADOWSOCKS", tableName: "APIAccess", value: "Shadowsocks", comment: "")
        case .socks5:
            NSLocalizedString("SOCKS_V5", tableName: "APIAccess", value: "Socks5", comment: "")
        }
    }

    /// Returns `true` if access method is configurable.
    /// Methods that aren't configurable do not offer any additional configuration.
    var hasProxyConfiguration: Bool {
        switch self {
        case .direct, .bridges:
            false
        case .shadowsocks, .socks5:
            true
        }
    }
}
