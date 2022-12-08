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
final class Tunnel: Equatable {
    /// Unique identifier assigned to instance at the time of creation.
    let identifier = UUID()

    #if DEBUG
    /// System VPN configuration identifier.
    /// This property performs a private call to obtain system configuration ID so it does not
    /// guarantee to return anything, also it may not return anything for newly created tunnels.
    var systemIdentifier: UUID? {
        let configurationKey = "configuration"
        let identifierKey = "identifier"

        guard tunnelProvider.responds(to: NSSelectorFromString(configurationKey)),
              let config = tunnelProvider.value(forKey: configurationKey) as? NSObject,
              config.responds(to: NSSelectorFromString(identifierKey)),
              let identifier = config.value(forKey: identifierKey) as? UUID
        else {
            return nil
        }

        return identifier
    }
    #endif

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

    func logFormat() -> String {
        var s = identifier.uuidString
        #if DEBUG
        if let configurationIdentifier = systemIdentifier?.uuidString {
            s += " (system profile ID: \(configurationIdentifier))"
        }
        #endif
        return s
    }

    private let lock = NSLock()
    private var observerList = ObserverList<TunnelStatusObserver>()

    private var _startDate: Date?
    private let tunnelProvider: TunnelProviderManagerType

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

    func setConfiguration(_ configuration: TunnelConfiguration) {
        configuration.apply(to: tunnelProvider)
    }

    func saveToPreferences(_ completion: @escaping (Error?) -> Void) {
        tunnelProvider.saveToPreferences { error in
            if let error = error {
                completion(error)
            } else {
                // Refresh connection status after saving the tunnel preferences.
                // Basically it's only necessary to do for new instances of
                // `NETunnelProviderManager`, but we do that for the existing ones too
                // for simplicity as it has no side effects.
                self.tunnelProvider.loadFromPreferences(completionHandler: completion)
            }
        }
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
