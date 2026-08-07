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

actor StartTunnelTask {
    private let interactor: TunnelInteractor
    private let logger = Logger(label: "StartTunnelTask")

    init(interactor: TunnelInteractor) {
        self.interactor = interactor
    }

    func start() async -> Result<Void, Error> {
        guard case .loggedIn = interactor.deviceState else {
            return .failure(InvalidDeviceStateError())
        }

        switch interactor.tunnelStatus.state {
        case .disconnecting(.nothing):
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnecting(.reconnect)
            }
            return .success(())

        case .disconnected, .pendingReconnect, .waitingForConnectivity:
            let result = await makeTunnelProviderAndStartTunnel()
            return result.map { .failure($0) } ?? .success(())

        default:
            return .success(())
        }
    }

    private func makeTunnelProviderAndStartTunnel() async -> Error? {
        let result = await makeTunnelProvider()

        do {
            try startTunnel(tunnel: result.get())
            return nil
        } catch {
            return error
        }
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
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .connecting(
                selectedRelays,
                isPostQuantum: isPostQuantum,
                isDaita: isDaita
            )
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider() async -> Result<any TunnelProtocol, Error> {
        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first ?? interactor.createNewTunnel()
        let configuration = TunnelConfiguration(
            includeAllNetworks: interactor.settings.includeAllNetworks.includeAllNetworksIsEnabled,
            excludeLocalNetworks: interactor.settings.includeAllNetworks.localNetworkSharingIsEnabled
        )

        return await withCheckedContinuation { continuation in
            tunnel.setConfiguration(configuration)
            tunnel.saveToPreferences { error in
                continuation.resume(returning: error.map { .failure($0) } ?? .success(tunnel))
            }
        }
    }
}
