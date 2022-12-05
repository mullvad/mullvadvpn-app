//
//  Tunnels.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit
import MullvadLogging

final class Tunnels {
    static let shared = Tunnels()

    private var nslock = NSLock()

    private var _tunnels: [Tunnel] = []

    var current: Tunnel? {
        nslock.lock()
        defer { nslock.unlock() }

        return _tunnels.first
    }

    private lazy var logger = Logger(label: "Tunnels")

    private init() {
        loadAllConfigurations()
    }

    private func loadAllConfigurations() {
        TunnelProviderManagerType.loadAllFromPreferences { tunnelProviders, error in
            if let error = error {
                self.logger.error(error: error,
                                  message: "Failed to load vpn configurations.")
                return
            }

            self._tunnels = tunnelProviders?.map(Tunnel.init(tunnelProvider:)) ?? []
        }
    }

    func store(
        tunnel: Tunnel,
        _ completionHandler: @escaping (Result<Tunnel, Error>) -> Void
    ) {
        tunnel.saveToPreferences { error in
            if let error = error {
                completionHandler(.failure(error))
            } else {
                // Refresh connection status after saving the tunnel preferences.
                // Basically it's only necessary to do for new instances of
                // `NETunnelProviderManager`, but we do that for the existing ones too
                // for simplicity as it has no side effects.
                tunnel.loadFromPreferences { error in
                    completionHandler(error.map { .failure($0) } ?? .success(tunnel))
                }
            }
        }
    }

    func refresh(
        tunnel: Tunnel,
        _ completionHandler: ((Result<Tunnel, Error>) -> Void)? = nil
    ) {
        tunnel.loadFromPreferences { error in
            completionHandler?(error.map { .failure($0) } ?? .success(tunnel))
        }
    }

    private func registerAppLifecycleObserverHandler() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(appEntersForegroundNotificationHandler),
            name: UIApplication.willEnterForegroundNotification,
            object: nil
        )
    }

    @objc private func appEntersForegroundNotificationHandler(_ notification: Notification) {
        _tunnels.forEach { refresh(tunnel: $0) }
    }
}
