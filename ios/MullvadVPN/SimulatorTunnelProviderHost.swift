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
import MullvadTypes
import enum NetworkExtension.NEProviderStopReason
import RelayCache
import RelaySelector
import TunnelProviderMessaging

class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {
    private var selectorResult: RelaySelectorResult?
    private let urlSession = REST.makeURLSession()
    private var proxiedRequests = [UUID: URLSessionDataTask]()
    private let relayCacheTracker: RelayCacheTracker

    private let providerLogger = Logger(label: "SimulatorTunnelProviderHost")
    private let dispatchQueue = DispatchQueue(label: "SimulatorTunnelProviderHostQueue")

    init(relayCacheTracker: RelayCacheTracker) {
        self.relayCacheTracker = relayCacheTracker
    }

    override func startTunnel(
        options: [String: NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        dispatchQueue.async {
            var selectorResult: RelaySelectorResult?

            do {
                let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])

                selectorResult = try tunnelOptions.getSelectorResult()
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: """
                    Failed to decode relay selector result passed from the app. \
                    Will continue by picking new relay.
                    """
                )
            }

            do {
                self.selectorResult = try selectorResult ?? self.pickRelay()

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

    override func stopTunnel(
        with reason: NEProviderStopReason,
        completionHandler: @escaping () -> Void
    ) {
        dispatchQueue.async {
            self.selectorResult = nil

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

    private func handleProviderMessage(
        _ message: TunnelProviderMessage,
        completionHandler: ((Data?) -> Void)?
    ) {
        switch message {
        case .getTunnelStatus:
            var tunnelStatus = PacketTunnelStatus()
            tunnelStatus.tunnelRelay = self.selectorResult?.packetTunnelRelay

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

        case let .reconnectTunnel(aSelectorResult):
            reasserting = true
            if let aSelectorResult = aSelectorResult {
                selectorResult = aSelectorResult
            }
            reasserting = false
            completionHandler?(nil)

        case let .sendURLRequest(proxyRequest):
            let task = urlSession
                .dataTask(with: proxyRequest.urlRequest) { [weak self] data, response, error in
                    guard let self = self else { return }

                    self.dispatchQueue.async {
                        self.proxiedRequests.removeValue(forKey: proxyRequest.id)

                        var reply: Data?
                        do {
                            let proxyResponse = ProxyURLResponse(
                                data: data,
                                response: response,
                                error: error
                            )
                            reply = try TunnelProviderReply(proxyResponse).encode()
                        } catch {
                            self.providerLogger.error(
                                error: error,
                                message: "Failed to encode ProxyURLResponse."
                            )
                        }

                        completionHandler?(reply)
                    }
                }

            proxiedRequests[proxyRequest.id] = task

            task.resume()

        case let .cancelURLRequest(id):
            let task = proxiedRequests.removeValue(forKey: id)

            task?.cancel()

            completionHandler?(nil)
        }
    }

    private func pickRelay() throws -> RelaySelectorResult {
        let cachedRelays = try relayCacheTracker.getCachedRelays()
        let tunnelSettings = try SettingsManager.readSettings()

        return try RelaySelector.evaluate(
            relays: cachedRelays.relays,
            constraints: tunnelSettings.relayConstraints
        )
    }
}

#endif
