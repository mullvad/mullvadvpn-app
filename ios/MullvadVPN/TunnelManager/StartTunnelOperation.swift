//
//  StartTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import NetworkExtension
import Operations
import PacketTunnelCore
import MullvadREST

class StartTunnelOperation: ResultOperation<Void> {
    typealias EncodeErrorHandler = (Error) -> Void

    private let interactor: TunnelInteractor
    private let logger = Logger(label: "StartTunnelOperation")

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        completionHandler: @escaping CompletionHandler
    ) {
        self.interactor = interactor

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

        case .disconnected, .pendingReconnect:
            makeTunnelProviderAndStartTunnel { error in
                self.finish(result: error.map { .failure($0) } ?? .success(()))
            }

        default:
            finish(result: .success(()))
        }
    }

    private func makeTunnelProviderAndStartTunnel(completionHandler: @escaping (Error?) -> Void) {
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
        let selectedRelay = try? interactor.selectRelay()
        var tunnelOptions = PacketTunnelOptions()

        do {
            if let selectedRelay {
                try tunnelOptions.setSelectedRelay(selectedRelay)
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
            tunnelStatus.state = .connecting(selectedRelay)
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider(completionHandler: @escaping (Result<any TunnelProtocol, Error>) -> Void) {
        let persistentTunnels = interactor.getPersistentTunnels()
        let tunnel = persistentTunnels.first ?? interactor.createNewTunnel()
        let configuration = Self.makeTunnelConfiguration()

        tunnel.setConfiguration(configuration)
        tunnel.saveToPreferences { error in
            completionHandler(error.map { .failure($0) } ?? .success(tunnel))
        }
    }

    private class func makeTunnelConfiguration() -> TunnelConfiguration {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationTarget.packetTunnel.bundleIdentifier
        protocolConfig.serverAddress = ""

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
