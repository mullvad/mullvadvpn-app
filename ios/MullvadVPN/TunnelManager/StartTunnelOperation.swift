//
//  StartTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

class StartTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    typealias EncodeErrorHandler = (Error) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State

    private var encodeErrorHandler: EncodeErrorHandler?

    init(queue: DispatchQueue, state: TunnelManager.State, encodeErrorHandler: @escaping EncodeErrorHandler, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.encodeErrorHandler = encodeErrorHandler

        super.init(completionQueue: queue, completionHandler: completionHandler)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            guard let tunnelInfo = self.state.tunnelInfo else {
                self.finish(completion: .failure(.unsetAccount))
                return
            }

            switch self.state.tunnelStatus.state {
            case .disconnecting(.nothing):
                self.state.tunnelStatus.state = .disconnecting(.reconnect)

                self.finish(completion: .success(()))

            case .disconnected, .pendingReconnect:
                RelayCache.Tracker.shared.read { readResult in
                    self.queue.async {
                        switch readResult {
                        case .success(let cachedRelays):
                            self.didReceiveRelays(
                                tunnelInfo: tunnelInfo,
                                cachedRelays: cachedRelays
                            )

                        case .failure(let error):
                            self.finish(completion: .failure(.readRelays(error)))
                        }
                    }
                }

            default:
                // Do not attempt to start the tunnel in all other cases.
                self.finish(completion: .success(()))
            }
        }
    }

    private func didReceiveRelays(tunnelInfo: TunnelInfo, cachedRelays: RelayCache.CachedRelays) {
        let selectorResult = RelaySelector.evaluate(
            relays: cachedRelays.relays,
            constraints: tunnelInfo.tunnelSettings.relayConstraints
        )

        guard let selectorResult = selectorResult else {
            finish(completion: .failure(.cannotSatisfyRelayConstraints))
            return
        }

        Self.makeTunnelProvider(accountToken: tunnelInfo.token) { makeTunnelProviderResult in
            self.queue.async {
                switch makeTunnelProviderResult {
                case .success(let tunnelProvider):
                    let startTunnelResult = Result { try self.startTunnel(tunnelProvider: tunnelProvider, selectorResult: selectorResult) }

                    self.finish(completion: OperationCompletion(result: startTunnelResult.mapError { .startVPNTunnel($0) }))

                case .failure(let error):
                    self.finish(completion: .failure(error))
                }
            }
        }
    }

    private func startTunnel(tunnelProvider: TunnelProviderManagerType, selectorResult: RelaySelectorResult) throws {
        var tunnelOptions = PacketTunnelOptions()

        do {
            try tunnelOptions.setSelectorResult(selectorResult)
        } catch {
            encodeErrorHandler?(error)
        }

        encodeErrorHandler = nil

        state.setTunnel(Tunnel(tunnelProvider: tunnelProvider), shouldRefreshTunnelState: false)
        state.tunnelStatus.reset(to: .connecting(selectorResult.packetTunnelRelay))

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
