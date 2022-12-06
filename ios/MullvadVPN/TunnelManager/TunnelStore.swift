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

/// Wrapper around system VPN tunnels.
final class TunnelStore: TunnelStatusObserver {
    private let logger = Logger(label: "TunnelStore")
    private let lock = NSLock()

    /// Persistent tunnels registered with the system.
    private var persistentTunnels: [Tunnel] = []

    /// Newly created tunnels, stored as collection of weak boxes.
    private var newTunnels: [WeakBox<Tunnel>] = []

    init(application: UIApplication) {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive(_:)),
            name: UIApplication.didBecomeActiveNotification,
            object: application
        )
    }

    func getPersistentTunnels() -> [Tunnel] {
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

    func createNewTunnel() -> Tunnel {
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

    func tunnel(_ tunnel: Tunnel, didReceiveStatus status: NEVPNStatus) {
        lock.lock()
        defer { lock.unlock() }

        handleTunnelStatus(tunnel: tunnel, status: status)
    }

    private func handleTunnelStatus(tunnel: Tunnel, status: NEVPNStatus) {
        if status == .invalid, let index = persistentTunnels.firstIndex(of: tunnel) {
            persistentTunnels.remove(at: index)
            logger.debug("Persistent tunnel was removed: \(tunnel.logFormat()).")
        }

        if status != .invalid, let index = newTunnels.firstIndex(where: { $0.value == tunnel }) {
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
