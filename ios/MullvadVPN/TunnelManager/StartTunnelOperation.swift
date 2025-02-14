//
//  StartTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import NetworkExtension
import Operations
import PacketTunnelCore

class StartTunnelOperation: ResultOperation<Void>, @unchecked Sendable {
    typealias EncodeErrorHandler = (Error) -> Void

    private let interactor: TunnelInteractor
    private let logger = Logger(label: "StartTunnelOperation")
    private let tunnelSettings: LatestTunnelSettings

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        tunnelSettings: LatestTunnelSettings,
        completionHandler: @escaping CompletionHandler
    ) {
        self.interactor = interactor
        self.tunnelSettings = tunnelSettings

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        guard case .loggedIn = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        switch interactor.tunnelStatus.state {
        case .disconnecting(.nothing):
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnecting(.reconnect)
            }

            finish(result: .success(()))

        case .disconnected, .pendingReconnect, .waitingForConnectivity:
            makeTunnelProviderAndStartTunnel { error in
                self.finish(result: error.map { .failure($0) } ?? .success(()))
            }

        default:
            finish(result: .success(()))
        }
    }

    private func makeTunnelProviderAndStartTunnel(completionHandler: @escaping @Sendable (Error?) -> Void) {
        makeTunnelProvider { result in
            self.dispatchQueue.async {
                do {
                    try self.startTunnel(tunnel: result.get())
                    completionHandler(nil)
                } catch {
                    completionHandler(error)
                }
            }
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

        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .connecting(
                selectedRelays,
                isPostQuantum: interactor.settings.tunnelQuantumResistance.isEnabled,
                isDaita: interactor.settings.daita.daitaState.isEnabled
            )
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider(
        completionHandler: @escaping @Sendable (Result<any TunnelProtocol, Error>)
            -> Void
    ) {
        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first ?? interactor.createNewTunnel()
        let configuration = makeTunnelConfiguration()

        tunnel.setConfiguration(configuration)
        tunnel.saveToPreferences { error in
            completionHandler(error.map { .failure($0) } ?? .success(tunnel))
        }
    }

    private func makeTunnelConfiguration() -> TunnelConfiguration {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationTarget.packetTunnel.bundleIdentifier
        protocolConfig.serverAddress = ""
        protocolConfig.includeAllNetworks = true
        protocolConfig.excludeLocalNetworks = tunnelSettings.excludeLocalNetwork
        #if DEBUG
        // Always exclude local networks to avoid killing the debugger immediately
        // when debugging the app
        protocolConfig.excludeLocalNetworks = true
        if #available(iOS 17.4, *) {
            protocolConfig.excludeDeviceCommunication = true
        }
        #endif

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
