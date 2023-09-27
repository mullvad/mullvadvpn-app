//
//  TunnelStore.swift
//  MullvadVPN
//
//  Created by pronebird on 07/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import NetworkExtension
import UIKit

protocol TunnelStoreProtocol {
    func getPersistentTunnels() -> [any TunnelProtocol]
    func createNewTunnel() -> any TunnelProtocol
}

/// Wrapper around system VPN tunnels.
final class TunnelStore: TunnelStoreProtocol, TunnelStatusObserver {
    private let logger = Logger(label: "TunnelStore")
    private let lock = NSLock()

    /// Persistent tunnels registered with the system.
    private var persistentTunnels: [any TunnelProtocol] = []

    /// Newly created tunnels, stored as collection of weak boxes.
    private var newTunnels: [WeakBox<any TunnelProtocol>] = []

    init(application: UIApplication) {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive(_:)),
            name: UIApplication.didBecomeActiveNotification,
            object: application
        )
    }

    func getPersistentTunnels() -> [any TunnelProtocol] {
        lock.lock()
        defer { lock.unlock() }

        return persistentTunnels
    }

    func loadPersistentTunnels(completion: @escaping (Error?) -> Void) {
        TunnelProviderManagerType.loadAllFromPreferences { managers, error in
            self.lock.lock()
            defer {
                self.lock.unlock()

                completion(error)
            }

            guard error == nil else { return }

            self.persistentTunnels.forEach { tunnel in
                tunnel.removeObserver(self)
            }

            self.persistentTunnels = managers?.map { manager in
                let tunnel = Tunnel(tunnelProvider: manager)
                tunnel.addObserver(self)

                self.logger.debug(
                    "Loaded persistent tunnel: \(tunnel.logFormat()) with status: \(tunnel.status)."
                )

                return tunnel
            } ?? []
        }
    }

    func createNewTunnel() -> any TunnelProtocol {
        lock.lock()
        defer { lock.unlock() }

        let tunnelProviderManager = TunnelProviderManagerType()
        let tunnel = Tunnel(tunnelProvider: tunnelProviderManager)
        tunnel.addObserver(self)

        newTunnels = newTunnels.filter { $0.value != nil }
        newTunnels.append(WeakBox(tunnel))

        logger.debug("Create new tunnel: \(tunnel.logFormat()).")

        return tunnel
    }

    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {
        lock.lock()
        defer { lock.unlock() }

        handleTunnelStatus(tunnel: tunnel, status: status)
    }

    private func handleTunnelStatus(tunnel: any TunnelProtocol, status: NEVPNStatus) {
        guard let tunnel = tunnel as? Tunnel else { return }

        if status == .invalid,
           let index = persistentTunnels.map({ $0 as? Tunnel }).firstIndex(of: tunnel) {
            persistentTunnels.remove(at: index)
            logger.debug("Persistent tunnel was removed: \(tunnel.logFormat()).")
        }

        if status != .invalid,
           let index = newTunnels.map({ $0.value as? Tunnel }).firstIndex(where: { $0 == tunnel }) {
            newTunnels.remove(at: index)
            persistentTunnels.append(tunnel)
            logger.debug("New tunnel became persistent: \(tunnel.logFormat()).")
        }
    }

    @objc private func applicationDidBecomeActive(_ notification: Notification) {
        refreshStatus()
    }

    private func refreshStatus() {
        lock.lock()
        defer { lock.unlock() }

        let allTunnels = persistentTunnels + newTunnels.compactMap { $0.value }

        for tunnel in allTunnels {
            handleTunnelStatus(tunnel: tunnel, status: tunnel.status)
        }
    }
}
