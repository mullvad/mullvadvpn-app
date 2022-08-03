//
//  SimulatorTunnelProviderHost.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

    import Foundation
    import Logging
    import enum NetworkExtension.NEProviderStopReason

    class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {
        private var selectorResult: RelaySelectorResult?

        private let providerLogger = Logger(label: "SimulatorTunnelProviderHost")
        private let dispatchQueue = DispatchQueue(label: "SimulatorTunnelProviderHostQueue")

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
                        chainedError: AnyChainedError(error),
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
                        chainedError: AnyChainedError(error),
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
                    let response = try self.processMessage(messageData)

                    completionHandler?(response)
                } catch {
                    self.providerLogger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to handle app message."
                    )

                    completionHandler?(nil)
                }
            }
        }

        private func processMessage(_ messageData: Data) throws -> Data? {
            let message = try TunnelProviderMessage(messageData: messageData)

            switch message {
            case .getTunnelStatus:
                var tunnelStatus = PacketTunnelStatus()
                tunnelStatus.tunnelRelay = self.selectorResult?.packetTunnelRelay

                return try TunnelProviderReply(tunnelStatus).encode()

            case let .reconnectTunnel(aSelectorResult):
                reasserting = true
                if let aSelectorResult = aSelectorResult {
                    selectorResult = aSelectorResult
                }
                reasserting = false

                return nil
            }
        }

        private func pickRelay() throws -> RelaySelectorResult {
            let cachedRelays = try RelayCache.Tracker.shared.getCachedRelays()
            let tunnelSettings = try SettingsManager.readSettings()

            return try RelaySelector.evaluate(
                relays: cachedRelays.relays,
                constraints: tunnelSettings.relayConstraints
            )
        }
    }

#endif
