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
import RelayCache
import RelaySelector
import TunnelProviderMessaging

class StartTunnelOperation: ResultOperation<Void, Error> {
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
            finish(completion: .failure(InvalidDeviceStateError()))
            return
        }

        switch interactor.tunnelStatus.state {
        case .disconnecting(.nothing):
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnecting(.reconnect)
            }

            finish(completion: .success(()))

        case .disconnected, .pendingReconnect:
            do {
                let selectorResult = try interactor.selectRelay()

                makeTunnelProviderAndStartTunnel(selectorResult: selectorResult) { error in
                    self.finish(completion: OperationCompletion(error: error))
                }
            } catch {
                finish(completion: .failure(error))
            }

        default:
            finish(completion: .success(()))
        }
    }

    private func makeTunnelProviderAndStartTunnel(
        selectorResult: RelaySelectorResult,
        completionHandler: @escaping (Error?) -> Void
    ) {
        makeTunnelProvider { result in
            self.dispatchQueue.async {
                do {
                    try self.startTunnel(
                        tunnel: try result.get(),
                        selectorResult: selectorResult
                    )

                    completionHandler(nil)
                } catch {
                    completionHandler(error)
                }
            }
        }
    }

    private func startTunnel(tunnel: Tunnel, selectorResult: RelaySelectorResult) throws {
        var tunnelOptions = PacketTunnelOptions()

        do {
            try tunnelOptions.setSelectorResult(selectorResult)
        } catch {
            logger.error(
                error: error,
                message: "Failed to encode the selector result."
            )
        }

        interactor.setTunnel(tunnel, shouldRefreshTunnelState: false)

        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.packetTunnelStatus.tunnelRelay = selectorResult.packetTunnelRelay
            tunnelStatus.state = .connecting(selectorResult.packetTunnelRelay)
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider(completionHandler: @escaping (Result<Tunnel, Error>) -> Void) {
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
        protocolConfig.providerBundleIdentifier = ApplicationConfiguration
            .packetTunnelExtensionIdentifier
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
