//
//  Tunnel.swift
//  MullvadVPN
//
//  Created by pronebird on 25/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

// Switch to stabs on simulator
#if targetEnvironment(simulator)
    typealias TunnelProviderManagerType = SimulatorTunnelProviderManager
#else
    typealias TunnelProviderManagerType = NETunnelProviderManager
#endif

protocol TunnelStatusObserver {
    func tunnel(_ tunnel: Tunnel, didReceiveStatus status: NEVPNStatus)
}

/// Tunnel wrapper class.
class Tunnel {
    /// Tunnel provider manager.
    private let tunnelProvider: TunnelProviderManagerType

    /// Tunnel start date.
    ///
    /// It's set to `distantPast` when the VPN connection was established prior to being observed
    /// by the class.
    var startDate: Date? {
        lock.lock()
        defer { lock.unlock() }

        return _startDate
    }

    /// Tunnel connection status.
    var status: NEVPNStatus {
        return tunnelProvider.connection.status
    }

    /// Whether on-demand VPN is enabled.
    var isOnDemandEnabled: Bool {
        get {
            return tunnelProvider.isOnDemandEnabled
        }
        set {
            tunnelProvider.isOnDemandEnabled = newValue
        }
    }

    private let lock = NSLock()
    private var observerList = ObserverList<TunnelStatusObserver>()

    private var _startDate: Date?

    init(tunnelProvider: TunnelProviderManagerType) {
        self.tunnelProvider = tunnelProvider

        NotificationCenter.default.addObserver(
            self, selector: #selector(handleVPNStatusChangeNotification(_:)),
            name: .NEVPNStatusDidChange,
            object: tunnelProvider.connection
        )

        handleVPNStatus(tunnelProvider.connection.status)
    }

    func start(options: [String: NSObject]?) throws {
        try tunnelProvider.connection.startVPNTunnel(options: options)
    }

    func stop() {
        tunnelProvider.connection.stopVPNTunnel()
    }

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        let session = tunnelProvider.connection as! VPNTunnelProviderSessionProtocol

        try session.sendProviderMessage(messageData, responseHandler: responseHandler)
    }

    func saveToPreferences(_ completion: @escaping (Error?) -> Void) {
        tunnelProvider.saveToPreferences(completionHandler: completion)
    }

    func removeFromPreferences(completion: @escaping (Error?) -> Void) {
        tunnelProvider.removeFromPreferences(completionHandler: completion)
    }

    func addBlockObserver(
        queue: DispatchQueue? = nil,
        handler: @escaping (Tunnel, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver {
        let observer = TunnelStatusBlockObserver(tunnel: self, queue: queue, handler: handler)

        addObserver(observer)

        return observer
    }

    func addObserver(_ observer: TunnelStatusObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: TunnelStatusObserver) {
        observerList.remove(observer)
    }

    @objc private func handleVPNStatusChangeNotification(_ notification: Notification) {
        guard let connection = notification.object as? VPNConnectionProtocol else { return }

        let newStatus = connection.status

        handleVPNStatus(newStatus)

        observerList.forEach { observer in
            observer.tunnel(self, didReceiveStatus: newStatus)
        }
    }

    private func handleVPNStatus(_ status: NEVPNStatus) {
        switch status {
        case .connecting:
            lock.lock()
            _startDate = Date()
            lock.unlock()

        case .connected, .reasserting:
            lock.lock()
            if _startDate == nil {
                _startDate = .distantPast
            }
            lock.unlock()

        case .disconnecting:
            break

        case .disconnected, .invalid:
            lock.lock()
            _startDate = nil
            lock.unlock()

        @unknown default:
            break
        }
    }
}

extension Tunnel: Equatable {
    static func == (lhs: Tunnel, rhs: Tunnel) -> Bool {
        return lhs.tunnelProvider == rhs.tunnelProvider
    }
}

final class TunnelStatusBlockObserver: TunnelStatusObserver {
    typealias Handler = (Tunnel, NEVPNStatus) -> Void

    private weak var tunnel: Tunnel?
    private let queue: DispatchQueue?
    private let handler: Handler

    fileprivate init(tunnel: Tunnel, queue: DispatchQueue?, handler: @escaping Handler) {
        self.tunnel = tunnel
        self.queue = queue
        self.handler = handler
    }

    func invalidate() {
        tunnel?.removeObserver(self)
    }

    func tunnel(_ tunnel: Tunnel, didReceiveStatus status: NEVPNStatus) {
        let block = {
            self.handler(tunnel, status)
        }

        if let queue = queue {
            queue.async(execute: block)
        } else {
            block()
        }
    }
}
