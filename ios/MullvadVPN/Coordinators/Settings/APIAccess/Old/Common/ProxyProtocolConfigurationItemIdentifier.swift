// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
