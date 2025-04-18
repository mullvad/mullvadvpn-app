//
//  Socks5Configuration.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Socks5 configuration.
/// - See: ``URLSessionSocks5Transport``
public struct Socks5Configuration: Equatable, Sendable {
    /// The socks proxy endpoint.
    public var proxyEndpoint: AnyIPEndpoint

    public var username: String?
    public var password: String?

    public init(proxyEndpoint: AnyIPEndpoint, username: String? = nil, password: String? = nil) {
        self.proxyEndpoint = proxyEndpoint
        self.username = username
        self.password = password
    }
}
