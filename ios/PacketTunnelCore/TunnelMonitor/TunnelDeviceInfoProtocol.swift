//
//  TunnelDeviceInfoProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol TunnelDeviceInfoProtocol {
    /// Returns tunnel interface name (i.e utun0) if available.
    var interfaceName: String? { get }

    /// Returns tunnel statistics.
    func getStats() throws -> WgStats
}
