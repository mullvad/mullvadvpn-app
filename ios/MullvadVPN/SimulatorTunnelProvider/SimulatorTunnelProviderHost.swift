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
import MullvadTypes
import NetworkExtension
import PacketTunnelCore

final class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {
    private var observedState: ObservedState = .disconnected
    private var selectedRelays: SelectedRelays?
    private let urlRequestProxy: URLRequestProxy
    private let relaySelector: RelaySelectorProtocol

    private let providerLogger = Logger(label: "SimulatorTunnelProviderHost")
    private let dispatchQueue = DispatchQueue(label: "SimulatorTunnelProviderHostQueue")

    init(relaySelector: RelaySelectorProtocol, transportProvider: TransportProvider) {
        self.relaySelector = relaySelector
        self.urlRequestProxy = URLRequestProxy(
            dispatchQueue: dispatchQueue,
            transportProvider: transportProvider
        )
    }

    override func startTunnel(
        options: [String: NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        dispatchQueue.async { [weak self] in
            guard let self else {
                completionHandler(nil)
                return
            }

            var selectedRelays: SelectedRelays?

            do {
                let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])

                selectedRelays = try tunnelOptions.getSelectedRelays()
            } catch {
                providerLogger.error(
                    error: error,
                    message: """
                    Failed to decode selected relay passed from the app. \
                    Will continue by picking new relay.
                    """
                )
            }

            do {
                setInternalStateConnected(with: try selectedRelays ?? pickRelays())
                completionHandler(nil)
            } catch {
                providerLogger.error(
                    error: error,
                    message: "Failed to pick relay."
                )
                completionHandler(error)
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        dispatchQueue.async { [weak self] in
            self?.selectedRelays = nil
            self?.observedState = .disconnected

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
            var reply: Data?
            do {
                reply = try TunnelProviderReply(observedState).encode()
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
            case let .preSelected(selectedRelays):
                self.selectedRelays = selectedRelays
            case .random:
                if let nextRelays = try? pickRelays() {
                    self.selectedRelays = nextRelays
                }
            case .current:
                break
            }

            setInternalStateConnected(with: selectedRelays)
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

        case let .cancelURLRequest(listId):
            urlRequestProxy.cancelRequest(identifier: listId)

            completionHandler?(nil)

        case .privateKeyRotation:
            completionHandler?(nil)
        }
    }

    private func pickRelays() throws -> SelectedRelays {
        let tunnelSettings = try SettingsManager.readSettings()

        return try relaySelector.selectRelays(
            with: tunnelSettings.relayConstraints,
            connectionAttemptCount: 0
        )
    }

    private func setInternalStateConnected(with selectedRelays: SelectedRelays?) {
        guard let selectedRelays = selectedRelays else { return }

        do {
            let settings = try SettingsManager.readSettings()
            observedState = .connected(
                ObservedConnectionState(
                    selectedRelays: selectedRelays,
                    relayConstraints: settings.relayConstraints,
                    networkReachability: .reachable,
                    connectionAttemptCount: 0,
                    transportLayer: .udp,
                    remotePort: selectedRelays.exit.endpoint.ipv4Relay.port, // TODO: Multihop
                    isPostQuantum: settings.tunnelQuantumResistance.isEnabled
                )
            )
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed to read device settings."
            )
        }
    }
}

#endif
