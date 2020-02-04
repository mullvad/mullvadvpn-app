//
//  SimulatorTunnelProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 05/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import NetworkExtension

// MARK: - NEPacketTunnelProvider mock

protocol SimulatorTunnelProviderDelegate {
    func startTunnel(options: [String: Any]?, completionHandler: @escaping (Error?) -> Void)
    func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void)
    func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?)
}

class SimulatorTunnelProvider {
    static let shared = SimulatorTunnelProvider()

    private let lock = NSLock()
    private var _delegate: SimulatorTunnelProviderDelegate?

    var delegate: SimulatorTunnelProviderDelegate! {
        get {
            lock.withCriticalBlock { _delegate }
        }
        set {
            lock.withCriticalBlock {
                _delegate = newValue
            }
        }
    }

    private init() {}

    fileprivate func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)? = nil) {
        self.delegate.handleAppMessage(messageData, completionHandler: completionHandler)
    }
}

// MARK: - NEVPNConnection mock

class SimulatorVPNConnection {

    private let lock = NSRecursiveLock()

    private(set) var status: NEVPNStatus {
        get {
            lock.withCriticalBlock { _status }
        }
        set {
            lock.withCriticalBlock {
                if newValue != _status {
                    _status = newValue
                    postStatusDidChangeNotification()
                }
            }
        }
    }
    private var _status: NEVPNStatus = .disconnected

    func startVPNTunnel(options: [String : Any]? = nil) throws {
        status = .connecting

        SimulatorTunnelProvider.shared.delegate.startTunnel(options: options) { (error) in
            if error == nil {
                self.status = .connected
            } else {
                self.status = .disconnected
            }
        }
    }

    func stopVPNTunnel() {
        status = .disconnecting

        SimulatorTunnelProvider.shared.delegate.stopTunnel(with: .none) {
            self.status = .disconnected
        }
    }

    private func postStatusDidChangeNotification() {
        NotificationCenter.default.post(name: .NEVPNStatusDidChange, object: self)
    }
}

// MARK: - NETunnelProviderSession mock

class SimulatorTunnelProviderSession: SimulatorVPNConnection {

    func startTunnel(options: [String : Any]?) throws {
        try startVPNTunnel(options: options)
    }

    func stopTunnel() {
        stopVPNTunnel()
    }

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        SimulatorTunnelProvider.shared.handleAppMessage(messageData, completionHandler: responseHandler)
    }

}

// MARK: - NETunnelProviderManager mock

struct SimulatorTunnelProviderManager: Equatable {

    static let tunnelLock = NSLock()
    static var tunnels = [SimulatorTunnelProviderManager]()

    private let lock = NSLock()
    private var _isEnabled = false
    private var _protocolConfiguration: NEVPNProtocol?
    private var _localizedDescription: String?

    private let identifier = UUID().uuidString

    var isEnabled: Bool {
        get {
            lock.withCriticalBlock { _isEnabled }
        }
        set {
            lock.withCriticalBlock {
                _isEnabled = newValue
            }
        }
    }
    var protocolConfiguration: NEVPNProtocol? {
        get {
            lock.withCriticalBlock { _protocolConfiguration }
        }
        set {
            lock.withCriticalBlock {
                _protocolConfiguration = newValue
            }
        }
    }

    var localizedDescription: String? {
        get {
            lock.withCriticalBlock { _localizedDescription }
        }
        set {
            lock.withCriticalBlock {
                _localizedDescription = newValue
            }
        }
    }

    let connection = SimulatorTunnelProviderSession()

    static func loadAllFromPreferences(completionHandler: ([SimulatorTunnelProviderManager]?, Error?) -> Void) {
        tunnelLock.withCriticalBlock {
            completionHandler(tunnels, nil)
        }
    }

    mutating func loadFromPreferences(completionHandler: (Error?) -> Void) {
        Self.tunnelLock.withCriticalBlock {
            if let savedTunnel = Self.tunnels.first(where: { $0.identifier == self.identifier }) {
                self.protocolConfiguration = savedTunnel.protocolConfiguration
                self.isEnabled = savedTunnel.isEnabled
                self.localizedDescription = savedTunnel.localizedDescription

                completionHandler(nil)
            } else {
                completionHandler(NEVPNError(.configurationInvalid))
            }

        }
    }

    func saveToPreferences(completionHandler: (Error?) -> Void) {
        Self.tunnelLock.withCriticalBlock {
            if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                Self.tunnels[index] = self
            } else {
                Self.tunnels.append(self)
            }

            completionHandler(nil)
        }
    }

    func removeFromPreferences(completionHandler: (Error?) -> Void) {
        Self.tunnelLock.withCriticalBlock {
            if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                Self.tunnels.remove(at: index)
                completionHandler(nil)
            } else {
                completionHandler(NEVPNError(.configurationReadWriteFailed))
            }
        }
    }

    static func == (lhs: SimulatorTunnelProviderManager, rhs: SimulatorTunnelProviderManager) -> Bool {
        lhs.identifier == rhs.identifier
    }

}

#endif
