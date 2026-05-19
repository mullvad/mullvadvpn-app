//
//  AccessMethodKind.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A kind of API access method.
public enum AccessMethodKind: Equatable, Hashable, CaseIterable {
    /// Direct communication.
    case direct

    /// Communication over bridges.
    case bridges

    /// Communication over proxy address from a DNS.
    case encryptedDNS

    /// Communication over shadowsocks.
    case shadowsocks

    /// Communication over socks v5 proxy.
    case socks5
}

public extension AccessMethodKind {
    /// Returns `true` if the method is permanent and cannot be deleted.
    var isPermanent: Bool {
        switch self {
        case .direct, .bridges, .encryptedDNS:
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
        case .direct, .bridges, .encryptedDNS:
            ""
        case .shadowsocks:
            NSLocalizedString("Shadowsocks", comment: "")
        case .socks5:
            NSLocalizedString("Socks5", comment: "").uppercased()
        }
    }

    /// Returns `true` if access method is configurable.
    /// Methods that aren't configurable do not offer any additional configuration.
    var hasProxyConfiguration: Bool {
        switch self {
        case .direct, .bridges, .encryptedDNS:
            false
        case .shadowsocks, .socks5:
            true
        }
    }
}

extension PersistentAccessMethod {
    /// A kind of access method.
    public var kind: AccessMethodKind {
        switch proxyConfiguration {
        case .direct:
            .direct
        case .bridges:
            .bridges
        case .encryptedDNS:
            .encryptedDNS
        case .shadowsocks:
            .shadowsocks
        case .socks5:
            .socks5
        }
    }
}
