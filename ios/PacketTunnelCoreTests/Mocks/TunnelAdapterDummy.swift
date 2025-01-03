//
//  TunnelAdapterDummy.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
@testable import WireGuardKitTypes

/// Dummy tunnel adapter that does nothing and reports no errors.
class TunnelAdapterDummy: TunnelAdapterProtocol, @unchecked Sendable {
    func startMultihop(
        entryConfiguration: TunnelAdapterConfiguration?,
        exitConfiguration: TunnelAdapterConfiguration,
        daita: DaitaConfiguration?
    ) async throws {}

    func start(configuration: TunnelAdapterConfiguration, daita: DaitaConfiguration?) async throws {}

    func stop() async throws {}

    func update(configuration: TunnelAdapterConfiguration) async throws {}
}
