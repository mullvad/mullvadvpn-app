// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import NetworkExtension
import Operations
import PacketTunnelCore

class StartTunnelOperation: ResultOperation<Void>, @unchecked Sendable {
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
        Task {
            guard case .loggedIn = await interactor.getDeviceState() else {
                finish(result: .failure(InvalidDeviceStateError()))
                return
            }

            switch await interactor.getTunnelStatus().state {
            case .disconnecting(.nothing):
                await interactor.updateTunnelStatus { tunnelStatus in
                    tunnelStatus = TunnelStatus()
                    tunnelStatus.state = .disconnecting(.reconnect)
                }
                finish(result: .success(()))

            case .disconnected, .pendingReconnect, .waitingForConnectivity:
                // Capture settings on internalQueue before entering async context.
                let settings = interactor.settings
                makeTunnelProviderAndStartTunnel(settings: settings) { error in
                    self.finish(result: error.map { .failure($0) } ?? .success(()))
                }

            default:
                finish(result: .success(()))
            }
        }
    }

    private func makeTunnelProviderAndStartTunnel(
        settings: LatestTunnelSettings,
        completionHandler: @escaping @Sendable (Error?) -> Void
    ) {
        makeTunnelProvider(settings: settings) { result in
            Task {
                do {
                    try await self.startTunnel(tunnel: result.get(), settings: settings)
                    completionHandler(nil)
                } catch {
                    completionHandler(error)
                }
            }
        }
    }

    private func startTunnel(tunnel: any TunnelProtocol, settings: LatestTunnelSettings) async throws {
        let selectedRelays = try? await interactor.selectRelays()
        var tunnelOptions = PacketTunnelOptions()

        do {
            if let selectedRelays {
                try tunnelOptions.setSelectedRelays(selectedRelays)
            }
        } catch {
            logger.error(
                error: error,
                message: "Failed to encode the selector result."
            )
        }

        await interactor.setTunnel(tunnel, shouldRefreshTunnelState: false)

        await interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .connecting(
                selectedRelays,
                isPostQuantum: settings.tunnelQuantumResistance.isEnabled,
                isDaita: settings.daita.isEnabled
            )
        }

        try tunnel.start(options: tunnelOptions.rawOptions())
    }

    private func makeTunnelProvider(
        settings: LatestTunnelSettings,
        completionHandler: @escaping @Sendable (Result<any TunnelProtocol, Error>) -> Void
    ) {
        Task {
            let tunnel: any TunnelProtocol
            if let persistentTunnel = await interactor.getPersistentTunnel() {
                tunnel = persistentTunnel
            } else {
                tunnel = await interactor.createNewTunnel()
            }

            let configuration = TunnelConfiguration(
                includeAllNetworks: settings.includeAllNetworks.includeAllNetworksIsEnabled,
                excludeLocalNetworks: settings.includeAllNetworks.localNetworkSharingIsEnabled
            )

            tunnel.setConfiguration(configuration)
            tunnel.saveToPreferences { error in
                completionHandler(error.map { .failure($0) } ?? .success(tunnel))
            }
        }
    }
}
