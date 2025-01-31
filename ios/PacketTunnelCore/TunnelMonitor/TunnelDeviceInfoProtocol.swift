//
//  TunnelDeviceInfoProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A type that can provide statistics and basic information about tunnel device.
public protocol TunnelDeviceInfoProtocol: Sendable {
    /// Returns tunnel interface name (i.e utun0) if available.
    var interfaceName: String? { get }

    /// Returns tunnel statistics.
    func getStats() async throws -> WgStats

    /// Returns tunnel statistics.
//    func getStats() throws -> WgStats
}
