//
//  UpdateTunnelConfigurationOperation.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-12-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import NetworkExtension
import Operations
import PacketTunnelCore

class UpdateTunnelConfigurationOperation: ResultOperation<Void> {
    typealias EncodeErrorHandler = (Error) -> Void

    private let logger = Logger(label: "UpdateTunnelOperation")
    private let interactor: TunnelInteractor

    init(interactor: TunnelInteractor, dispatchQueue: DispatchQueue, completionHandler: @escaping CompletionHandler) {
        self.interactor = interactor

        super.init(dispatchQueue: dispatchQueue, completionQueue: dispatchQueue, completionHandler: completionHandler)
    }

    override func main() {
        guard case .loggedIn = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        switch interactor.tunnelStatus.state {
        case .disconnected, .disconnecting:
            finish(result: .success(()))

        case .connected, .error, .waitingForConnectivity, .connecting, .reconnecting, .negotiatingEphemeralPeer,
             .pendingReconnect:
            makeTunnelProviderAndUpdateTunnel { error in
                self.finish(result: error.map { .failure($0) } ?? .success(()))
            }
        }
    }

    private func makeTunnelProviderAndUpdateTunnel(completionHandler: @escaping (Error?) -> Void) {
        updateTunnelProvider { result in
            self.dispatchQueue.async {
                do {
                    try self.updateTunnel(tunnel: result.get())
                } catch {
                    completionHandler(error)
                }
            }
        }
    }

    private func updateTunnel(tunnel: any TunnelProtocol) throws {
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

        interactor.setTunnel(tunnel, shouldRefreshTunnelState: true)

        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .connecting(
                selectedRelays,
                isPostQuantum: interactor.settings.tunnelQuantumResistance.isEnabled,
                isDaita: interactor.settings.daita.daitaState.isEnabled
            )
        }

        _ = tunnel.reconnectTunnel(to: .current) { [weak self] result in
            self?.finish(result: result)
        }
    }

    private func updateTunnelProvider(completionHandler: @escaping (Result<any TunnelProtocol, Error>) -> Void) {
        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first!
        let configuration = Self.makeTunnelConfiguration()

//        tunnel.removeFromPreferences(completion: { _ in })
        tunnel.setConfiguration(configuration)
        tunnel.updatePreferences { error in
            completionHandler(error.map { .failure($0) } ?? .success(tunnel))
        }
    }

    private class func makeTunnelConfiguration() -> TunnelConfiguration {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationTarget.packetTunnel.bundleIdentifier
        protocolConfig.serverAddress = ""

        let includeAllNetworks = UserDefaults.standard.bool(forKey: "includeAllNetworks")
        let excludeLocalNetworks = UserDefaults.standard.bool(forKey: "excludeLocalNetworks")

        protocolConfig.includeAllNetworks = includeAllNetworks
        protocolConfig.excludeLocalNetworks = excludeLocalNetworks

        let alwaysOnRule = NEOnDemandRuleConnect()
        alwaysOnRule.interfaceTypeMatch = .any

        return TunnelConfiguration(
            isEnabled: true,
            localizedDescription: "WireGuard",
            protocolConfiguration: protocolConfig,
            onDemandRules: [alwaysOnRule],
            isOnDemandEnabled: true
        )
    }
}
