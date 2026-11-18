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
import MullvadTypes
@preconcurrency import NetworkExtension
import UIKit

protocol TunnelStoreProtocol: Sendable & Actor {
    associatedtype TunnelType: TunnelProtocol
    func getPersistentTunnel() -> TunnelType?
    func createNewTunnel() async -> TunnelType
}

/// Wrapper around system VPN tunnels.
actor TunnelStore: TunnelStoreProtocol, TunnelStatusObserver {
    typealias BackgroundTaskProvidingObject = BackgroundTaskProviding & AnyObject

    typealias TunnelType = Tunnel
    private let logger = Logger(label: "TunnelStore")
    private let application: BackgroundTaskProviding

    /// Persistent tunnels registered with the system.
    private var persistentTunnel: TunnelType?

    init(application: BackgroundTaskProvidingObject) {
        self.application = application
        NotificationCenter.default.addObserver(
            forName: UIApplication.didBecomeActiveNotification,
            object: application,
            queue: .main
        ) { [weak self] notification in
            Task {
                await self?.refreshStatus()
            }
        }
    }

    func getPersistentTunnel() -> TunnelType? {
        persistentTunnel
    }

    private func setPersistentTunnel(from manager: TunnelProviderManagerType) async {
        await persistentTunnel?.removeObserver(self)
        let tunnel = await Tunnel(tunnelProvider: manager, backgroundTaskProvider: self.application)
        await tunnel.addObserver(self)
        let status = await tunnel.status
        let tunnelIdentifier = await tunnel.logFormat()

        self.logger.debug(
            "Loaded persistent tunnel: \(tunnelIdentifier) with status: \(status)."
        )
        persistentTunnel = tunnel
    }

    func loadPersistentTunnels() async throws {
        guard let manager = try await TunnelProviderManagerType.loadAllFromPreferences().first else { return }
        await setPersistentTunnel(from: manager)
    }

    func createNewTunnel() async -> TunnelType {
        let tunnelProviderManager = TunnelProviderManagerType()
        let tunnel = await TunnelType(tunnelProvider: tunnelProviderManager, backgroundTaskProvider: application)
        await tunnel.addObserver(self)
        let tunnelIdentifier = await tunnel.logFormat()

        logger.debug("Create new tunnel: \(tunnelIdentifier).")

        return tunnel
    }

    nonisolated func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {
        Task {
            await handleTunnelStatus(tunnel: tunnel as! TunnelType, status: status)
        }
    }

    private func handleTunnelStatus(tunnel: TunnelType, status: NEVPNStatus) async {
        let tunnelIdentifier = await tunnel.logFormat()
        if status == .invalid {
            persistentTunnel = nil
            logger.debug("Persistent tunnel was removed: \(tunnelIdentifier).")
        }

        if status != .invalid {
            persistentTunnel = tunnel
            logger.debug("New tunnel became persistent: \(tunnelIdentifier).")
        }
    }

    private func refreshStatus() async {
        guard let persistentTunnel else { return }
        await handleTunnelStatus(tunnel: persistentTunnel, status: persistentTunnel.status)
    }
}
