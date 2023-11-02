//
//  ProxyProtocolConfigurationItemIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Item identifier used by diffable data sources implementing proxy configuration.
enum ProxyProtocolConfigurationItemIdentifier: Hashable {
    case socks(SocksItemIdentifier)
    case shadowsocks(ShadowsocksItemIdentifier)

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case let .shadowsocks(itemIdentifier):
            itemIdentifier.cellIdentifier
        case let .socks(itemIdentifier):
            itemIdentifier.cellIdentifier
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        switch self {
        case let .shadowsocks(itemIdentifier):
            itemIdentifier.isSelectable
        case let .socks(itemIdentifier):
            itemIdentifier.isSelectable
        }
    }
}
