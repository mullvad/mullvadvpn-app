//
//  ShadowsocksItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Item identifier used by diffable data sources implementing shadowsocks configuration.
enum ShadowsocksItemIdentifier: Hashable, CaseIterable {
    case server
    case port
    case password
    case cipher

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .server, .port, .password:
            .textInput
        case .cipher:
            .textWithDisclosure
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        self == .cipher
    }

    /// The text describing the item identifier and suitable to be used as a field label.
    var text: String {
        switch self {
        case .server:
            NSLocalizedString("SHADOWSOCKS_SERVER", tableName: "APIAccess", value: "Server", comment: "")
        case .port:
            NSLocalizedString("SHADOWSOCKS_PORT", tableName: "APIAccess", value: "Port", comment: "")
        case .password:
            NSLocalizedString("SHADOWSOCKS_PASSWORD", tableName: "APIAccess", value: "Password", comment: "")
        case .cipher:
            NSLocalizedString("SHADOWSOCKS_CIPHER", tableName: "APIAccess", value: "Cipher", comment: "")
        }
    }
}
