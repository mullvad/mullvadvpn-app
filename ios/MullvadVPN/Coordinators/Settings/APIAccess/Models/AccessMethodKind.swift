//
//  AccessMethodKind.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

/// A kind of API access method.
enum AccessMethodKind: Equatable, Hashable, CaseIterable {
    /// Direct communication.
    case direct

    /// Communication over bridges.
    case bridges

    /// Communication over shadowsocks.
    case shadowsocks

    /// Communication over socks v5 proxy.
    case socks5

    /// Returns `true` if the method is permanent and cannot be deleted.
    var isPermanent: Bool {
        switch self {
        case .direct, .bridges:
            true
        case .shadowsocks, .socks5:
            false
        }
    }

    /// Returns all access method kinds that can be added by user.
    static var allUserDefinedKinds: [AccessMethodKind] {
        allCases.filter { !$0.isPermanent }
    }

    /// Returns localized description describing the access method.
    var localizedDescription: String {
        switch self {
        case .direct, .bridges:
            ""
        case .shadowsocks:
            NSLocalizedString("SHADOWSOCKS", tableName: "APIAccess", value: "Shadowsocks", comment: "")
        case .socks5:
            NSLocalizedString("SOCKS_V5", tableName: "APIAccess", value: "Socks5", comment: "").uppercased()
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

extension PersistentAccessMethod {
    /// A kind of access method.
    var kind: AccessMethodKind {
        switch proxyConfiguration {
        case .direct:
            .direct
        case .bridges:
            .bridges
        case .shadowsocks:
            .shadowsocks
        case .socks5:
            .socks5
        }
    }
}
