//
//  StartTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import Logging

protocol StartTunnelOperationDelegate {
    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo?
    func operationDidRequestTunnelState(_ operation: Operation) -> TunnelState
    func operation(_ operation: Operation, didSetTunnelState newTunnelState: TunnelState)
    func operation(_ operation: Operation, didSetTunnelProvider newTunnelProvider: TunnelProviderManagerType)
}

class StartTunnelOperation: AsyncOperation {
    private let queue: DispatchQueue
    private let delegate: StartTunnelOperationDelegate

    private let logger = Logger(label: "TunnelManager.StartTunnelOperation")

    init(queue: DispatchQueue, delegate: StartTunnelOperationDelegate) {
        self.queue = queue
        self.delegate = delegate
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.logger.debug("Cancelled")
                self.finish()
                return
            }

            guard let tunnelInfo = self.delegate.operationDidRequestTunnelInfo(self) else {
                let tunnelManagerError = TunnelManager.Error.missingAccount
                self.logger.error(chainedError: tunnelManagerError)

                self.finish()
                return
            }

            let tunnelState = self.delegate.operationDidRequestTunnelState(self)

            switch tunnelState {
            case .disconnecting(.nothing):
                self.delegate.operation(self, didSetTunnelState: .disconnecting(.reconnect))
                self.finish()

            case .disconnected, .pendingReconnect:
                RelayCache.Tracker.shared.read { readResult in
                    self.queue.async {
                        switch readResult {
                        case .success(let cachedRelays):
                            self.didReceiveRelays(tunnelInfo: tunnelInfo, cachedRelays: cachedRelays) { error in
                                if let error = error {
                                    self.logger.error(chainedError: error)
                                }
                                self.finish()
                            }

                        case .failure(let error):
                            let tunnelManagerError = TunnelManager.Error.readRelays(error)
                            self.logger.error(chainedError: tunnelManagerError)
                            self.finish()
                        }
                    }
                }

            default:
                // Do not attempt to start the tunnel in all other cases.
                self.finish()
            }
        }
    }

    private func didReceiveRelays(tunnelInfo: TunnelInfo, cachedRelays: RelayCache.CachedRelays, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let selectorResult = RelaySelector.evaluate(
            relays: cachedRelays.relays,
            constraints: tunnelInfo.tunnelSettings.relayConstraints
        )

        guard let selectorResult = selectorResult else {
            completionHandler(.cannotSatisfyRelayConstraints)
            return
        }

        Self.makeTunnelProvider(accountToken: tunnelInfo.token) { makeTunnelProviderResult in
            self.queue.async {
                guard case .success(let tunnelProvider) = makeTunnelProviderResult else {
                    completionHandler(makeTunnelProviderResult.error)
                    return
                }

                do {
                    try self.startTunnel(tunnelProvider: tunnelProvider, selectorResult: selectorResult)

                    completionHandler(nil)
                } catch {
                    completionHandler(.startVPNTunnel(error))
                }
            }
        }
    }

    private func startTunnel(tunnelProvider: TunnelProviderManagerType, selectorResult: RelaySelectorResult) throws {
        var tunnelOptions = PacketTunnelOptions()

        do {
            try tunnelOptions.setSelectorResult(selectorResult)
        } catch {
            self.logger.error(chainedError: AnyChainedError(error), message: "Failed to encode relay selector result.")
        }

        delegate.operation(self, didSetTunnelProvider: tunnelProvider)
        delegate.operation(self, didSetTunnelState: .connecting(selectorResult.tunnelConnectionInfo))

        try tunnelProvider.connection.startVPNTunnel(options: tunnelOptions.rawOptions())
    }

    private class func makeTunnelProvider(accountToken: String, completionHandler: @escaping (Result<TunnelProviderManagerType, TunnelManager.Error>) -> Void) {
        TunnelProviderManagerType.loadAllFromPreferences { tunnelProviders, error in
            if let error = error {
                completionHandler(.failure(.loadAllVPNConfigurations(error)))
                return
            }

            let result = Self.setupTunnelProvider(
                accountToken: accountToken,
                tunnels: tunnelProviders
            )

            guard case .success(let tunnelProvider) = result else {
                completionHandler(result)
                return
            }

            tunnelProvider.saveToPreferences { error in
                if let error = error {
                    completionHandler(.failure(.saveVPNConfiguration(error)))
                    return
                }

                // Refresh connection status after saving the tunnel preferences.
                // Basically it's only necessary to do for new instances of
                // `NETunnelProviderManager`, but we do that for the existing ones too
                // for simplicity as it has no side effects.
                tunnelProvider.loadFromPreferences { error in
                    if let error = error {
                        completionHandler(.failure(.reloadVPNConfiguration(error)))
                    } else {
                        completionHandler(.success(tunnelProvider))
                    }
                }
            }
        }
    }

    private class func setupTunnelProvider(accountToken: String, tunnels: [TunnelProviderManagerType]?) -> Result<TunnelProviderManagerType, TunnelManager.Error> {
        // Request persistent keychain reference to tunnel settings
        return TunnelSettingsManager.getPersistentKeychainReference(account: accountToken)
            .mapError { error in
                return .obtainPersistentKeychainReference(error)
            }
            .map { passwordReference in
                // Get the first available tunnel or make a new one
                let tunnelProvider = tunnels?.first ?? TunnelProviderManagerType()

                let protocolConfig = NETunnelProviderProtocol()
                protocolConfig.providerBundleIdentifier = ApplicationConfiguration.packetTunnelExtensionIdentifier
                protocolConfig.serverAddress = ""
                protocolConfig.username = accountToken
                protocolConfig.passwordReference = passwordReference

                tunnelProvider.isEnabled = true
                tunnelProvider.localizedDescription = "WireGuard"
                tunnelProvider.protocolConfiguration = protocolConfig

                // Enable on-demand VPN, always connect the tunnel when on Wi-Fi or cellular
                let alwaysOnRule = NEOnDemandRuleConnect()
                alwaysOnRule.interfaceTypeMatch = .any
                tunnelProvider.onDemandRules = [alwaysOnRule]
                tunnelProvider.isOnDemandEnabled = true

                return tunnelProvider
            }
    }
}
