//
//  ProxyConfigurationItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ProxyConfigurationItemIdentifier: Hashable {
    case `protocol`
    case proxyConfiguration(ProxyProtocolConfigurationItemIdentifier)

    /// Returns all shadowsocks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static var allShadowsocksItems: [ProxyConfigurationItemIdentifier] {
        ShadowsocksItemIdentifier.allCases.map { .proxyConfiguration(.shadowsocks($0)) }
    }

    /// Returns all socks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static func allSocksItems(authenticate: Bool) -> [ProxyConfigurationItemIdentifier] {
        SocksItemIdentifier.allCases(authenticate: authenticate).map { .proxyConfiguration(.socks($0)) }
    }

    /// Cell identifiers for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .protocol:
            .textWithDisclosure
        case let .proxyConfiguration(itemIdentifier):
            itemIdentifier.cellIdentifier
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case .protocol:
            true
        case let .proxyConfiguration(itemIdentifier):
            itemIdentifier.isSelectable
        }
    }

    /// The text label for the corresponding cell.
    var text: String? {
        switch self {
        case .protocol:
            NSLocalizedString("TYPE", tableName: "APIAccess", value: "Type", comment: "")
        case .proxyConfiguration:
            nil
        }
    }
}
