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
import NetworkExtension
import UIKit

protocol TunnelStoreProtocol: Sendable {
    associatedtype TunnelType: TunnelProtocol, Equatable
    func getPersistentTunnel() -> TunnelType?
    func createNewTunnel() -> TunnelType
}

/// Wrapper around system VPN tunnels.
final class TunnelStore: TunnelStoreProtocol, TunnelStatusObserver, @unchecked Sendable {
    typealias BackgroundTaskProvidingObject = BackgroundTaskProviding & AnyObject

    typealias TunnelType = Tunnel
    private let logger = Logger(label: "TunnelStore")
    private let lock = NSLock()
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
            self?.refreshStatus()
        }
    }

    func getPersistentTunnel() -> TunnelType? {
        lock.withLock {
            persistentTunnel
        }
    }

    private func setPersistentTunnel(from manager: TunnelProviderManagerType) {
        lock.withLock {
            persistentTunnel?.removeObserver(self)
            let tunnel = Tunnel(tunnelProvider: manager, backgroundTaskProvider: self.application)
            tunnel.addObserver(self)

            self.logger.debug(
                "Loaded persistent tunnel: \(tunnel.logFormat()) with status: \(tunnel.status)."
            )
            persistentTunnel = tunnel
        }
    }

    func loadPersistentTunnels() async throws {
        guard let manager = try await TunnelProviderManagerType.loadAllFromPreferences().first else { return }
        self.setPersistentTunnel(from: manager)
    }

    func createNewTunnel() -> TunnelType {
        lock.withLock {
            let tunnelProviderManager = TunnelProviderManagerType()
            let tunnel = TunnelType(tunnelProvider: tunnelProviderManager, backgroundTaskProvider: application)
            tunnel.addObserver(self)

            logger.debug("Create new tunnel: \(tunnel.logFormat()).")

            return tunnel
        }
    }

    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {
        lock.withLock {
            handleTunnelStatus(tunnel: tunnel as! TunnelType, status: status)
        }
    }

    private func handleTunnelStatus(tunnel: TunnelType, status: NEVPNStatus) {
        if status == .invalid {
            persistentTunnel = nil
            logger.debug("Persistent tunnel was removed: \(tunnel.logFormat()).")
        }

        if status != .invalid {
            persistentTunnel = tunnel
            logger.debug("New tunnel became persistent: \(tunnel.logFormat()).")
        }
    }

    private func refreshStatus() {
        lock.withLock {
            guard let persistentTunnel else { return }
            handleTunnelStatus(tunnel: persistentTunnel, status: persistentTunnel.status)
        }
    }
}
