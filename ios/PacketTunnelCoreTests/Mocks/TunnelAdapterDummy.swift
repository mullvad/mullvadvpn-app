//
//  TunnelAdapterDummy.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

/// Dummy tunnel adapter that does nothing and reports no errors.
class TunnelAdapterDummy: TunnelAdapterProtocol {
    func startPostQuantumKeyExchange(configuration: PacketTunnelCore.TunnelAdapterConfiguration) async throws {}

    func start(configuration: TunnelAdapterConfiguration) async throws {}

    func stop() async throws {}

    func update(configuration: TunnelAdapterConfiguration) async throws {}
}
