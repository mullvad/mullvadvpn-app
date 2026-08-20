//
//  TunnelStore.swift
//  MullvadVPN
//
//  Created by pronebird on 07/12/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import NetworkExtension
import UIKit

protocol TunnelStoreProtocol: Sendable {
    associatedtype TunnelType: TunnelProtocol, Equatable
    func getPersistentTunnels() -> [TunnelType]
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
    private var persistentTunnels: [TunnelType] = []

    /// Newly created tunnels, stored as collection of weak boxes.
    private var newTunnels: [WeakBox<TunnelType>] = []

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

    func getPersistentTunnels() -> [TunnelType] {
        lock.withLock {
            persistentTunnels
        }
    }

    private func setPersistentTunnelsFromManagers(_ managers: [TunnelProviderManagerType]) {
        lock.withLock {
            self.persistentTunnels.forEach { tunnel in
                tunnel.removeObserver(self)
            }

            self.persistentTunnels =
                managers.map { manager in
                    let tunnel = Tunnel(tunnelProvider: manager, backgroundTaskProvider: self.application)
                    tunnel.addObserver(self)

                    self.logger.debug(
                        "Loaded persistent tunnel: \(tunnel.logFormat()) with status: \(tunnel.status)."
                    )

                    return tunnel
                }
        }
    }

    func loadPersistentTunnels() async throws {
        let managers = try await TunnelProviderManagerType.loadAllFromPreferences()
        self.setPersistentTunnelsFromManagers(managers)
    }

    func createNewTunnel() -> TunnelType {
        lock.withLock {
            let tunnelProviderManager = TunnelProviderManagerType()
            let tunnel = TunnelType(tunnelProvider: tunnelProviderManager, backgroundTaskProvider: application)
            tunnel.addObserver(self)

            newTunnels = newTunnels.filter { $0.value != nil }
            newTunnels.append(WeakBox(tunnel))

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
        if status == .invalid,
            let index = persistentTunnels.firstIndex(of: tunnel)
        {
            persistentTunnels.remove(at: index)
            logger.debug("Persistent tunnel was removed: \(tunnel.logFormat()).")
        }

        if status != .invalid,
            let index = newTunnels.compactMap({ $0.value }).firstIndex(where: { $0 == tunnel })
        {
            newTunnels.remove(at: index)
            persistentTunnels.append(tunnel)
            logger.debug("New tunnel became persistent: \(tunnel.logFormat()).")
        }
    }

    private func refreshStatus() {
        lock.withLock {
            let allTunnels = persistentTunnels + newTunnels.compactMap { $0.value }

            for tunnel in allTunnels {
                handleTunnelStatus(tunnel: tunnel, status: tunnel.status)
            }
        }
    }
}
