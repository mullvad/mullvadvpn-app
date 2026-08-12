//
//  StartTunnelTask.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 03/08/2026.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import PacketTunnelCore

final class StartTunnelTask: Sendable {
    private let interactor: TunnelInteractor
    private let logger = Logger(label: "StartTunnelTask")

    init(interactor: TunnelInteractor) {
        self.interactor = interactor
    }

    func start() async throws {
        try Task.checkCancellation()
        guard case .loggedIn = interactor.deviceState else {
            throw InvalidDeviceStateError()
        }

        switch interactor.tunnelStatus.state {
        case .disconnecting(.nothing):
            interactor.updateTunnelStatus { tunnelStatus in
                //if !Task.isCancelled {
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnecting(.reconnect)
                //}
            }

        case .disconnected, .pendingReconnect, .waitingForConnectivity:
            try await makeTunnelProviderAndStartTunnel()
        default:
            break
        }
    }

    private func makeTunnelProviderAndStartTunnel() async throws {
        let tunnel = try await makeTunnelProvider()
        try Task.checkCancellation()
        try startTunnel(tunnel: tunnel)
    }

    private func startTunnel(tunnel: any TunnelProtocol) throws {
        let selectedRelays = try? interactor.selectRelays()
        var tunnelOptions = PacketTunnelOptions()

        do {
            if let selectedRelays {
                try tunnelOptions.setSelectedRelays(selectedRelays)
            }
        } catch {
            logger.error(
                error: error,
                message: "Failed to encode the selector result."
            )
        }

        interactor.setTunnel(tunnel, shouldRefreshTunnelState: false)

        let isPostQuantum = interactor.settings.tunnelQuantumResistance.isEnabled
        let isDaita = interactor.settings.daita.isEnabled

        interactor.updateTunnelStatus { tunnelStatus in
            //if !Task.isCancelled {
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .connecting(
                selectedRelays,
                isPostQuantum: isPostQuantum,
                isDaita: isDaita
            )
            //}
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider() async throws -> any TunnelProtocol {
        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first ?? interactor.createNewTunnel()
        let configuration = TunnelConfiguration(
            includeAllNetworks: interactor.settings.includeAllNetworks.includeAllNetworksIsEnabled,
            excludeLocalNetworks: interactor.settings.includeAllNetworks.localNetworkSharingIsEnabled
        )
        tunnel.setConfiguration(configuration)
        try await saveToPreferences(tunnel)
        try Task.checkCancellation()
        return tunnel
    }

    private func saveToPreferences(_ tunnel: any TunnelProtocol) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            tunnel.saveToPreferences { error in
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(returning: ())
            }
        }
    }
}
