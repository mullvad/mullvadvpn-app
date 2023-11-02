//
//  AddAccessMethodItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum AddAccessMethodItemIdentifier: Hashable {
    case name
    case `protocol`
    case proxyConfiguration(ProxyProtocolConfigurationItemIdentifier)

    /// Returns all shadowsocks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static var allShadowsocksItems: [AddAccessMethodItemIdentifier] {
        ShadowsocksItemIdentifier.allCases.map { .proxyConfiguration(.shadowsocks($0)) }
    }

    /// Returns all socks items wrapped into `ProxyConfigurationItemIdentifier.proxyConfiguration`.
    static func allSocksItems(authenticate: Bool) -> [AddAccessMethodItemIdentifier] {
        SocksItemIdentifier.allCases(authenticate: authenticate).map { .proxyConfiguration(.socks($0)) }
    }

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .name:
            .textInput
        case .protocol:
            .textWithDisclosure
        case let .proxyConfiguration(item):
            item.cellIdentifier
        }
    }

    /// Whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case .name:
            false
        case .protocol:
            true
        case let .proxyConfiguration(item):
            item.isSelectable
        }
    }

    /// The text label for the corresponding cell.
    var text: String? {
        switch self {
        case .name:
            NSLocalizedString("NAME", tableName: "APIAccess", value: "Name", comment: "")
        case .protocol:
            NSLocalizedString("TYPE", tableName: "APIAccess", value: "Type", comment: "")
        case .proxyConfiguration:
            nil
        }
    }
}
