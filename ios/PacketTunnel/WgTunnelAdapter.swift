//
//  TunnelAdapterProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import NetworkExtension
import WireGuardKit

class WgTunnelAdapter: TunnelAdapterProtocol {
    private let wgAdapter: WireGuardAdapter

    init(_ packetTunnelProvider: NEPacketTunnelProvider) {
        let tunnelLogger = Logger(label: "WireGuard")

        wgAdapter = WireGuardAdapter(
            with: packetTunnelProvider,
            shouldHandleReasserting: false,
            logHandler: { logLevel, message in
                tunnelLogger.log(level: logLevel.loggerLevel, "\(message)")
            }
        )
    }

    func start(configuration: TunnelConfiguration) async throws {
        try await wgAdapter.start(tunnelConfiguration: configuration)
    }

    func stop() async throws {
        try await wgAdapter.stop()
    }

    func update(configuration: TunnelConfiguration) async throws {
        try await wgAdapter.update(tunnelConfiguration: configuration)
    }
}
