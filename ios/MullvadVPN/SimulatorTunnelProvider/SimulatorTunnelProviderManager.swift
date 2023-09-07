//
//  SimulatorTunnelProviderManager.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-09-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import NetworkExtension

final class SimulatorTunnelProviderManager: NSObject, VPNTunnelProviderManagerProtocol {
    static let tunnelsLock = NSRecursiveLock()
    fileprivate static var tunnels = [SimulatorTunnelInfo]()

    private let lock = NSLock()
    private var tunnelInfo: SimulatorTunnelInfo
    private var identifier: String {
        lock.lock()
        defer { lock.unlock() }

        return tunnelInfo.identifier
    }

    var isOnDemandEnabled: Bool {
        get {
            lock.lock()
            defer { lock.unlock() }

            return tunnelInfo.isOnDemandEnabled
        }
        set {
            lock.lock()
            tunnelInfo.isOnDemandEnabled = newValue
            lock.unlock()
        }
    }

    var onDemandRules: [NEOnDemandRule] {
        get {
            lock.lock()
            defer { lock.unlock() }

            return tunnelInfo.onDemandRules
        }
        set {
            lock.lock()
            tunnelInfo.onDemandRules = newValue
            lock.unlock()
        }
    }

    var isEnabled: Bool {
        get {
            lock.lock()
            defer { lock.unlock() }

            return tunnelInfo.isEnabled
        }
        set {
            lock.lock()
            tunnelInfo.isEnabled = newValue
            lock.unlock()
        }
    }

    var protocolConfiguration: NEVPNProtocol? {
        get {
            lock.lock()
            defer { lock.unlock() }

            return tunnelInfo.protocolConfiguration
        }
        set {
            lock.lock()
            tunnelInfo.protocolConfiguration = newValue
            lock.unlock()
        }
    }

    var localizedDescription: String? {
        get {
            lock.lock()
            defer { lock.unlock() }

            return tunnelInfo.localizedDescription
        }
        set {
            lock.lock()
            tunnelInfo.localizedDescription = newValue
            lock.unlock()
        }
    }

    var connection: SimulatorVPNConnection {
        lock.lock()
        defer { lock.unlock() }

        return tunnelInfo.connection
    }

    static func loadAllFromPreferences(completionHandler: (
        [SimulatorTunnelProviderManager]?,
        Error?
    ) -> Void) {
        Self.tunnelsLock.lock()
        let tunnelProviders = tunnels.map { tunnelInfo in
            SimulatorTunnelProviderManager(tunnelInfo: tunnelInfo)
        }
        Self.tunnelsLock.unlock()

        completionHandler(tunnelProviders, nil)
    }

    override required init() {
        tunnelInfo = SimulatorTunnelInfo()
        super.init()
    }

    private init(tunnelInfo: SimulatorTunnelInfo) {
        self.tunnelInfo = tunnelInfo
        super.init()
    }

    func loadFromPreferences(completionHandler: (Error?) -> Void) {
        var error: NEVPNError?

        Self.tunnelsLock.lock()

        if let savedTunnel = Self.tunnels.first(where: { $0.identifier == self.identifier }) {
            tunnelInfo = savedTunnel
        } else {
            error = NEVPNError(.configurationInvalid)
        }

        Self.tunnelsLock.unlock()

        completionHandler(error)
    }

    func saveToPreferences(completionHandler: ((Error?) -> Void)?) {
        Self.tunnelsLock.lock()

        if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
            Self.tunnels[index] = tunnelInfo
        } else {
            Self.tunnels.append(tunnelInfo)
        }

        Self.tunnelsLock.unlock()

        completionHandler?(nil)
    }

    func removeFromPreferences(completionHandler: ((Error?) -> Void)?) {
        var error: NEVPNError?

        Self.tunnelsLock.lock()

        if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
            Self.tunnels.remove(at: index)
        } else {
            error = NEVPNError(.configurationReadWriteFailed)
        }

        Self.tunnelsLock.unlock()

        completionHandler?(error)
    }

    override func isEqual(_ object: Any?) -> Bool {
        guard let other = object as? Self else { return false }
        return self.identifier == other.identifier
    }
}

#endif
