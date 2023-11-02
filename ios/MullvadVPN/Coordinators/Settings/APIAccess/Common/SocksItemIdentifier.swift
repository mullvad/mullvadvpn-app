//
//  SocksItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Item identifier used by diffable data sources implementing socks configuration.
enum SocksItemIdentifier: Hashable, CaseIterable {
    case server
    case port
    case authentication
    case username
    case password

    /// Compute item identifiers that should be present in the diffable data source.
    ///
    /// - Parameter authenticate: whether user opt-in for socks proxy authentication.
    /// - Returns: item identifiers to display in the diffable data source.
    static func allCases(authenticate: Bool) -> [Self] {
        allCases.filter { itemIdentifier in
            if authenticate {
                return true
            } else {
                return itemIdentifier != .username && itemIdentifier != .password
            }
        }
    }

    /// Returns cell identifier for the item identiifer.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .server, .username, .password, .port:
            .textInput
        case .authentication:
            .toggle
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        false
    }

    /// The text describing the item identifier and suitable to be used as a field label.
    var text: String {
        switch self {
        case .server:
            NSLocalizedString("SOCKS_SERVER", tableName: "APIAccess", value: "Server", comment: "")
        case .port:
            NSLocalizedString("SOCKS_PORT", tableName: "APIAccess", value: "Port", comment: "")
        case .authentication:
            NSLocalizedString("SOCKS_AUTHENTICATION", tableName: "APIAccess", value: "Authentication", comment: "")
        case .username:
            NSLocalizedString("SOCKS_USERNAME", tableName: "APIAccess", value: "Username", comment: "")
        case .password:
            NSLocalizedString("SOCKS_PASSWORD", tableName: "APIAccess", value: "Password", comment: "")
        }
    }
}
