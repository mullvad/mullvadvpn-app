//
//  PacketTunnelStatus.swift
//  TunnelProviderMessaging
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Struct describing packet tunnel process status.
public struct PacketTunnelStatus: Codable, Equatable {
    /// Last tunnel error.
    public var lastError: String?

    /// Flag indicating whether network is reachable.
    public var isNetworkReachable: Bool

    /// Current relay.
    public var tunnelRelay: PacketTunnelRelay?

    public init(
        lastError: String? = nil,
        isNetworkReachable: Bool = true,
        tunnelRelay: PacketTunnelRelay? = nil
    ) {
        self.lastError = lastError
        self.isNetworkReachable = isNetworkReachable
        self.tunnelRelay = tunnelRelay
    }
}
