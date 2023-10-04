//
//  SimulatorTunnelProviderHost.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTransport
import MullvadTypes
import enum NetworkExtension.NEProviderStopReason
import PacketTunnelCore
import RelayCache
import RelaySelector

final class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {
    private var selectedRelay: SelectedRelay?
    private let urlRequestProxy: URLRequestProxy
    private let relayCacheTracker: RelayCacheTracker

    private let providerLogger = Logger(label: "SimulatorTunnelProviderHost")
    private let dispatchQueue = DispatchQueue(label: "SimulatorTunnelProviderHostQueue")

    init(relayCacheTracker: RelayCacheTracker, transportProvider: TransportProvider) {
        self.relayCacheTracker = relayCacheTracker
        self.urlRequestProxy = URLRequestProxy(
            dispatchQueue: dispatchQueue,
            transportProvider: transportProvider
        )
    }

    override func startTunnel(
        options: [String: NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        dispatchQueue.async {
            var selectedRelay: SelectedRelay?

            do {
                let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])

                selectedRelay = try tunnelOptions.getSelectedRelay()
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: """
                    Failed to decode selected relay passed from the app. \
                    Will continue by picking new relay.
                    """
                )
            }

            do {
                self.selectedRelay = try selectedRelay ?? self.pickRelay()

                completionHandler(nil)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to pick relay."
                )
                completionHandler(error)
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        dispatchQueue.async {
            self.selectedRelay = nil

            completionHandler()
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            do {
                let message = try TunnelProviderMessage(messageData: messageData)

                self.handleProviderMessage(message, completionHandler: completionHandler)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to decode app message."
                )

                completionHandler?(nil)
            }
        }
    }

    private func handleProviderMessage(_ message: TunnelProviderMessage, completionHandler: ((Data?) -> Void)?) {
        switch message {
        case .getTunnelStatus:
            var tunnelStatus = PacketTunnelStatus()
            tunnelStatus.tunnelRelay = self.selectedRelay?.packetTunnelRelay

            var reply: Data?
            do {
                reply = try TunnelProviderReply(tunnelStatus).encode()
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to encode tunnel status."
                )
            }

            completionHandler?(reply)

        case let .reconnectTunnel(nextRelay):
            reasserting = true
            switch nextRelay {
            case let .preSelected(selectedRelay):
                self.selectedRelay = selectedRelay
            case .random:
                if let nextRelay = try? pickRelay() {
                    self.selectedRelay = nextRelay
                }
            case .current:
                break
            }
            reasserting = false
            completionHandler?(nil)

        case let .sendURLRequest(proxyRequest):
            urlRequestProxy.sendRequest(proxyRequest) { response in
                var reply: Data?
                do {
                    reply = try TunnelProviderReply(response).encode()
                } catch {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to encode ProxyURLResponse."
                    )
                }
                completionHandler?(reply)
            }

        case let .cancelURLRequest(id):
            urlRequestProxy.cancelRequest(identifier: id)

            completionHandler?(nil)

        case .privateKeyRotation:
            completionHandler?(nil)
        }
    }

    private func pickRelay() throws -> SelectedRelay {
        let cachedRelays = try relayCacheTracker.getCachedRelays()
        let tunnelSettings = try SettingsManager.readSettings()
        let selectorResult = try RelaySelector.evaluate(
            relays: cachedRelays.relays,
            constraints: tunnelSettings.relayConstraints,
            numberOfFailedAttempts: 0
        )
        return SelectedRelay(
            endpoint: selectorResult.endpoint,
            hostname: selectorResult.relay.hostname,
            location: selectorResult.location
        )
    }
}

#endif
