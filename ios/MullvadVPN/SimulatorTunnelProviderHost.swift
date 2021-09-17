//
//  SimulatorTunnelProviderHost.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import Network
import NetworkExtension
import Logging

class SimulatorTunnelProviderHost: SimulatorTunnelProviderDelegate {

    private var connectionInfo: TunnelConnectionInfo?
    private let providerLogger = Logger(label: "SimulatorTunnelProviderHost")
    private let stateQueue = DispatchQueue(label: "SimulatorTunnelProviderHostQueue")

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        stateQueue.async {
            let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])
            let appSelectorResult = Result { try tunnelOptions.getSelectorResult() }

            if let error = appSelectorResult.error {
                self.providerLogger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to decode relay selector result passed from the app. Will continue by picking new relay."
                )
            }

            if let appSelectorResult = appSelectorResult.flattenValue {
                self.connectionInfo = appSelectorResult.tunnelConnectionInfo
            } else {
                self.connectionInfo = self.pickRelay()?.tunnelConnectionInfo
            }

            completionHandler(nil)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        stateQueue.async {
            self.connectionInfo = nil

            completionHandler()
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        Result { try TunnelIPC.Coding.decodeRequest(messageData) }
            .asPromise()
            .receive(on: stateQueue)
            .onFailure { error in
                self.providerLogger.error(chainedError: AnyChainedError(error), message: "Failed to decode the IPC request.")
            }
            .success()
            .mapThen(defaultValue: nil) { request in
                switch request {
                case .tunnelConnectionInfo:
                    return Result { try TunnelIPC.Coding.encodeResponse(self.connectionInfo) }
                        .asPromise()
                        .onFailure { error in
                            self.providerLogger.error(chainedError: AnyChainedError(error), message: "Failed to encode tunnel connection info IPC response.")
                        }
                        .success()

                case .reloadTunnelSettings:
                    self.reasserting = true
                    self.connectionInfo = self.pickRelay()?.tunnelConnectionInfo
                    self.reasserting = false

                    return .resolved(nil)
                }
            }
            .observe { completion in
                completionHandler?(completion.unwrappedValue ?? nil)
            }
    }

    private func pickRelay() -> RelaySelectorResult? {
        guard let result = RelayCache.Tracker.shared.read().await().unwrappedValue else { return nil }

        switch result {
        case .success(let cachedRelays):
            let keychainReference = self.protocolConfiguration.passwordReference!

            switch TunnelSettingsManager.load(searchTerm: .persistentReference(keychainReference)) {
            case .success(let entry):
                return RelaySelector.evaluate(
                    relays: cachedRelays.relays,
                    constraints: entry.tunnelSettings.relayConstraints
                )
            case .failure(let error):
                self.providerLogger.error(chainedError: error, message: "Failed to load tunnel settings when picking relay")

                return nil
            }

        case .failure(let error):
            self.providerLogger.error(chainedError: error, message: "Failed to read relays when picking relay")
            return nil
        }
    }

}

#endif
