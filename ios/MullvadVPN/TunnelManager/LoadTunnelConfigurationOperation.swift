//
//  LoadTunnelConfigurationOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class LoadTunnelConfigurationOperation: ResultOperation<(), TunnelManager.Error> {
    private let logger = Logger(label: "LoadTunnelConfigurationOperation")
    private let state: TunnelManager.State

    init(dispatchQueue: DispatchQueue, state: TunnelManager.State) {
        self.state = state

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        TunnelProviderManagerType.loadAllFromPreferences { tunnels, error in
            self.dispatchQueue.async {
                if let error = error {
                    self.finish(completion: .failure(.loadAllVPNConfigurations(error)))
                } else {
                    self.didLoadVPNConfigurations(tunnels: tunnels)
                }
            }
        }
    }

    private func didLoadVPNConfigurations(tunnels: [TunnelProviderManagerType]?) {
        var returnError: TunnelManager.Error?
        var tunnelSettings: TunnelSettingsV2?
        do {
            tunnelSettings = try SettingsManager.readSettings()
        } catch .itemNotFound as KeychainError  {
            logger.debug("Settings not found in keychain.")
        } catch let error as DecodingError {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Cannot decode settings. Will attempt to delete them from keychain."
            )

            do {
                try SettingsManager.deleteSettings()
            } catch {
                returnError = .deleteSettings(error)

                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to delete settings from keychain."
                )
            }
        } catch {
            returnError = .readSettings(error)

            logger.error(
                chainedError: AnyChainedError(error),
                message: "Unexpected error when reading settings."
            )
        }

        let tunnel = tunnels?.first.map { tunnelProvider in
            return Tunnel(tunnelProvider: tunnelProvider)
        }

        if let tunnelSettings = tunnelSettings {
            state.tunnelSettings = tunnelSettings
            state.setTunnel(tunnel, shouldRefreshTunnelState: true)
            state.isLoadedConfiguration = true

            finish(completion: .success(()))
        } else {
            let onFinish = {
                self.state.tunnelSettings = nil
                self.state.setTunnel(nil, shouldRefreshTunnelState: true)
                self.state.isLoadedConfiguration = returnError == nil

                self.finish(completion: returnError.map { .failure($0) } ?? .success(()))
            }

            if let tunnel = tunnel {
                logger.debug("Remove orphaned VPN configuration.")

                tunnel.removeFromPreferences { error in
                    self.dispatchQueue.async {
                        if let error = error {
                            self.logger.error(
                                chainedError: AnyChainedError(error),
                                message: "Failed to remove VPN configuration."
                            )
                        }
                        onFinish()
                    }
                }
            } else {
                onFinish()
            }
        }
    }
}
