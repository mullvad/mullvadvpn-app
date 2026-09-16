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
import MullvadTypes
import NetworkExtension

// Switch to stabs on simulator
#if targetEnvironment(simulator)
    typealias TunnelProviderManagerType = SimulatorTunnelProviderManager
#else
    typealias TunnelProviderManagerType = NETunnelProviderManager
#endif

protocol TunnelStatusObserver: Sendable {
    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus)
}

protocol TunnelProtocol: AnyObject, Sendable {
    associatedtype TunnelManagerProtocol: VPNTunnelProviderManagerProtocol
    var status: NEVPNStatus { get async }
    var isOnDemandEnabled: Bool { get async }
    var startDate: Date? { get async }
    var backgroundTaskProvider: BackgroundTaskProviding { get }

    init(tunnelProvider: TunnelManagerProtocol, backgroundTaskProvider: BackgroundTaskProviding)

    func setOnDemandEnabled(enabled: Bool) async
    func addObserver(_ observer: any TunnelStatusObserver)
    func removeObserver(_ observer: any TunnelStatusObserver)
    func addBlockObserver(
        queue: DispatchQueue?,
        handler: @escaping (any TunnelProtocol, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver

    func logFormat() -> String

    func saveToPreferences() async -> Error?
    func removeFromPreferences() async -> Error?

    func setConfiguration(_ configuration: TunnelConfiguration) async
    func start(options: sending [String: NSObject]?) async throws
    func stop() async
    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws
}

/// Tunnel wrapper class.
actor Tunnel: TunnelProtocol, Equatable, @unchecked Sendable {
    /// Unique identifier assigned to instance at the time of creation.
    let identifier = UUID()

    let backgroundTaskProvider: BackgroundTaskProviding

    #if DEBUG
        /// System VPN configuration identifier.
        /// This property performs a private call to obtain system configuration ID so it does not
        /// guarantee to return anything, also it may not return anything for newly created tunnels.
        nonisolated var systemIdentifier: UUID? {
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
    var startDate: Date?

    /// Tunnel connection status.
    var status: NEVPNStatus {
        tunnelProvider.connection.status
    }

    /// Whether on-demand VPN is enabled.
    var isOnDemandEnabled: Bool {
        get {
            tunnelProvider.isOnDemandEnabled
        }
        set {
            tunnelProvider.isOnDemandEnabled = newValue
        }
    }

    private let observerList = ObserverList<any TunnelStatusObserver>()
    private nonisolated(unsafe) var notificationObserver: NSObjectProtocol?
    internal let tunnelProvider: TunnelProviderManagerType

    init(tunnelProvider: TunnelProviderManagerType, backgroundTaskProvider: BackgroundTaskProviding) {
        self.tunnelProvider = tunnelProvider
        self.backgroundTaskProvider = backgroundTaskProvider

        // Observe ALL NEVPNStatusDidChange notifications rather than filtering by specific
        // connection object. This is necessary because `loadFromPreferences` may internally
        // replace the connection object, causing notifications to be sent to a different
        // object than the one we originally registered for. We filter in the handler instead
        // by comparing against the current `tunnelProvider.connection`.
        notificationObserver = NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange,
            object: nil,
            queue: nil
        ) { [weak self] notification in
            guard let connection = notification.object as? VPNConnectionProtocol else { return }
            Task { await self?.handleVPNStatusChangeNotification(connection: connection) }
        }

        Task {
            await handleVPNStatus(tunnelProvider.connection.status)
        }
    }

    func setOnDemandEnabled(enabled: Bool) async {
        tunnelProvider.isOnDemandEnabled = enabled
    }

    nonisolated func logFormat() -> String {
        var s = identifier.uuidString
        #if DEBUG
            if let configurationIdentifier = systemIdentifier?.uuidString {
                s += " (system profile ID: \(configurationIdentifier))"
            }
        #endif
        return s
    }

    func start(options: sending [String: NSObject]?) throws {
        try tunnelProvider.connection.startVPNTunnel(options: options)
    }

    func stop() {
        tunnelProvider.connection.stopVPNTunnel()
    }

    nonisolated func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        let session = tunnelProvider.connection as? VPNTunnelProviderSessionProtocol

        try session?.sendProviderMessage(messageData, responseHandler: responseHandler)
    }

    func setConfiguration(_ configuration: TunnelConfiguration) {
        configuration.apply(to: tunnelProvider)
    }

    func saveToPreferences() async -> Error? {
        do {
            try await tunnelProvider.saveToPreferences()
            // Refresh connection status after saving the tunnel preferences.
            // Basically it's only necessary to do for new instances of
            // `NETunnelProviderManager`, but we do that for the existing ones too
            // for simplicity as it has no side effects.
            try await tunnelProvider.loadFromPreferences()
        } catch {
            return error
        }

        return nil
    }

    func removeFromPreferences() async -> Error? {
        do {
            try await tunnelProvider.removeFromPreferences()
        } catch {
            return error
        }

        return nil
    }

    nonisolated func addBlockObserver(
        queue: DispatchQueue? = nil,
        handler: @escaping (any TunnelProtocol, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver {
        let observer = TunnelStatusBlockObserver(tunnel: self, queue: queue, handler: handler)

        addObserver(observer)

        return observer
    }

    // Safe as long as `observerList.append` keeps its locks.
    nonisolated func addObserver(_ observer: any TunnelStatusObserver) {
        observerList.append(observer)
    }

    // Safe as long as `observerList.remove` keeps its locks.
    nonisolated func removeObserver(_ observer: any TunnelStatusObserver) {
        observerList.remove(observer)
    }

    private func handleVPNStatusChangeNotification(connection: VPNConnectionProtocol) async {
        // Filter to only handle notifications for our connection.
        // We compare against the current `tunnelProvider.connection` (not a captured reference)
        // because `loadFromPreferences` may replace the connection object internally.
        guard connection === tunnelProvider.connection else { return }

        let newStatus = connection.status

        handleVPNStatus(newStatus)

        observerList.notify { observer in
            observer.tunnel(self, didReceiveStatus: newStatus)
        }
    }

    private func handleVPNStatus(_ status: NEVPNStatus) {
        switch status {
        case .connecting:
            startDate = Date()

        case .connected, .reasserting:
            if startDate == nil {
                startDate = .distantPast
            }

        case .disconnecting:
            break

        case .disconnected, .invalid:
            startDate = nil

        @unknown default:
            break
        }
    }

    nonisolated static func == (lhs: Tunnel, rhs: Tunnel) -> Bool {
        lhs.tunnelProvider == rhs.tunnelProvider
    }
}
