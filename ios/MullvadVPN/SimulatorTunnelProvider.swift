//
//  SimulatorTunnelProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 05/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//


import Foundation
import NetworkExtension

// MARK: - Formal conformances

protocol VPNConnectionProtocol: NSObject {
    var status: NEVPNStatus { get }

    func startVPNTunnel() throws
    func startVPNTunnel(options: [String: NSObject]?) throws
    func stopVPNTunnel()
}

protocol VPNTunnelProviderSessionProtocol {
    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws
}

protocol VPNTunnelProviderManagerProtocol: Equatable {
    associatedtype SelfType: VPNTunnelProviderManagerProtocol
    associatedtype ConnectionType: VPNConnectionProtocol

    var isEnabled: Bool { get set }
    var protocolConfiguration: NEVPNProtocol? { get set }
    var localizedDescription: String? { get set }
    var connection: ConnectionType { get }

    init()

    func loadFromPreferences(completionHandler: @escaping (Error?) -> Void)
    func saveToPreferences(completionHandler: ((Error?) -> Void)?)
    func removeFromPreferences(completionHandler: ((Error?) -> Void)?)

    static func loadAllFromPreferences(completionHandler: @escaping ([SelfType]?, Error?) -> Void)
}

extension NEVPNConnection: VPNConnectionProtocol {}
extension NETunnelProviderSession: VPNTunnelProviderSessionProtocol {}
extension NETunnelProviderManager: VPNTunnelProviderManagerProtocol {}

extension VPNTunnelProviderManagerProtocol {
    static func loadAllFromPreferences() -> Result<[SelfType]?, Error>.Promise {
        return Result<[SelfType]?, Error>.Promise { resolver in
            Self.loadAllFromPreferences { tunnels, error in
                if let error = error {
                    resolver.resolve(value: .failure(error))
                } else {
                    resolver.resolve(value: .success(tunnels))
                }
            }
        }
    }

    func loadFromPreferences() -> Result<(), Error>.Promise {
        return Result<(), Error>.Promise { resolver in
            loadFromPreferences { error in
                if let error = error {
                    resolver.resolve(value: .failure(error))
                } else {
                    resolver.resolve(value: .success(()))
                }
            }
        }
    }

    func saveToPreferences() -> Result<(), Error>.Promise {
        return Result<(), Error>.Promise { resolver in
            saveToPreferences { error in
                if let error = error {
                    resolver.resolve(value: .failure(error))
                } else {
                    resolver.resolve(value: .success(()))
                }
            }
        }
    }

    func removeFromPreferences() -> Result<(), Error>.Promise {
        return Result<(), Error>.Promise { resolver in
            removeFromPreferences { error in
                if let error = error {
                    resolver.resolve(value: .failure(error))
                } else {
                    resolver.resolve(value: .success(()))
                }
            }
        }
    }
}

#if targetEnvironment(simulator)

// MARK: - NEPacketTunnelProvider stubs

class SimulatorTunnelProviderDelegate {
    fileprivate(set) var connection: SimulatorVPNConnection?

    var protocolConfiguration: NEVPNProtocol {
        return connection?.protocolConfiguration ?? NEVPNProtocol()
    }

    var reasserting: Bool {
        get {
            return connection?.reasserting ?? false
        }
        set {
            connection?.reasserting = newValue
        }
    }

    func startTunnel(options: [String: Any]?, completionHandler: @escaping (Error?) -> Void) {
        completionHandler(nil)
    }

    func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        completionHandler()
    }

    func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        completionHandler?(nil)
    }
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

// MARK: - NEVPNConnection stubs

class SimulatorVPNConnection: NSObject, VPNConnectionProtocol {

    // Protocol configuration is automatically synced by `SimulatorTunnelInfo`
    fileprivate var protocolConfiguration = NEVPNProtocol()

    private let lock = NSRecursiveLock()

    private var _status: NEVPNStatus = .disconnected
    private(set) var status: NEVPNStatus {
        get {
            lock.withCriticalBlock { _status }
        }
        set {
            lock.withCriticalBlock {
                if newValue != _status {
                    _status = newValue

                    // Send notification while holding the lock. This should enable the receiver
                    // to fetch the `SimulatorVPNConnection.status` before it changes.
                    postStatusDidChangeNotification()
                }
            }
        }
    }

    private var statusBeforeReasserting: NEVPNStatus?
    private var _reasserting = false
    var reasserting: Bool {
        get {
            lock.withCriticalBlock { _reasserting }
        }
        set {
            lock.withCriticalBlock {
                if newValue != _reasserting {
                    _reasserting = newValue

                    if newValue {
                        statusBeforeReasserting = status
                        status = .reasserting
                    } else if let newStatus = statusBeforeReasserting {
                        status = newStatus
                        statusBeforeReasserting = nil
                    }
                }
            }
        }
    }

    func startVPNTunnel() throws {
        try startVPNTunnel(options: nil)
    }

    func startVPNTunnel(options: [String: NSObject]?) throws {
        SimulatorTunnelProvider.shared.delegate.connection = self

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

// MARK: - NETunnelProviderSession stubs

class SimulatorTunnelProviderSession: SimulatorVPNConnection, VPNTunnelProviderSessionProtocol {

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        SimulatorTunnelProvider.shared.handleAppMessage(messageData, completionHandler: responseHandler)
    }

}

// MARK: - NETunnelProviderManager stubs

/// A mock struct for tunnel configuration and connection
private struct SimulatorTunnelInfo {
    /// A unique identifier for the configuration
    var identifier = UUID().uuidString

    /// An associated VPN connection.
    /// Intentionally initialized with a `SimulatorTunnelProviderSession` subclass which
    /// implements the necessary protocol
    var connection: SimulatorVPNConnection = SimulatorTunnelProviderSession()

    /// Whether configuration is enabled
    var isEnabled = false

    /// Whether on-demand VPN is enabled
    var isOnDemandEnabled = false

    /// On-demand VPN rules
    var onDemandRules = [NEOnDemandRule]()

    /// Protocol configuration
    var protocolConfiguration: NEVPNProtocol? {
        didSet {
            self.connection.protocolConfiguration = protocolConfiguration ?? NEVPNProtocol()
        }
    }

    /// Tunnel description
    var localizedDescription: String?

    /// Designated initializer
    init() {}
}

class SimulatorTunnelProviderManager: VPNTunnelProviderManagerProtocol, Equatable {

    static let tunnelsLock = NSRecursiveLock()
    fileprivate static var tunnels = [SimulatorTunnelInfo]()

    private let lock = NSLock()
    private var tunnelInfo: SimulatorTunnelInfo
    private var identifier: String {
        lock.withCriticalBlock { tunnelInfo.identifier }
    }

    var isOnDemandEnabled: Bool {
        get {
            lock.withCriticalBlock { tunnelInfo.isOnDemandEnabled }
        }
        set {
            lock.withCriticalBlock {
                tunnelInfo.isOnDemandEnabled = newValue
            }
        }
    }

    var onDemandRules: [NEOnDemandRule] {
        get {
            lock.withCriticalBlock { tunnelInfo.onDemandRules }
        }
        set {
            lock.withCriticalBlock { tunnelInfo.onDemandRules = newValue }
        }
    }

    var isEnabled: Bool {
        get {
            lock.withCriticalBlock { tunnelInfo.isEnabled }
        }
        set {
            lock.withCriticalBlock {
                tunnelInfo.isEnabled = newValue
            }
        }
    }

    var protocolConfiguration: NEVPNProtocol? {
        get {
            lock.withCriticalBlock { tunnelInfo.protocolConfiguration }
        }
        set {
            lock.withCriticalBlock {
                tunnelInfo.protocolConfiguration = newValue
            }
        }
    }

    var localizedDescription: String? {
        get {
            lock.withCriticalBlock { tunnelInfo.localizedDescription }
        }
        set {
            lock.withCriticalBlock {
                tunnelInfo.localizedDescription = newValue
            }
        }
    }

    var connection: SimulatorVPNConnection {
        lock.withCriticalBlock { tunnelInfo.connection }
    }

    static func loadAllFromPreferences(completionHandler: ([SimulatorTunnelProviderManager]?, Error?) -> Void) {
        tunnelsLock.withCriticalBlock {
            completionHandler(tunnels.map { SimulatorTunnelProviderManager(tunnelInfo: $0) }, nil)
        }
    }

    required convenience init() {
        self.init(tunnelInfo: SimulatorTunnelInfo())
    }

    private init(tunnelInfo: SimulatorTunnelInfo) {
        self.tunnelInfo = tunnelInfo
    }

    func loadFromPreferences(completionHandler: (Error?) -> Void) {
        Self.tunnelsLock.withCriticalBlock {
            if let savedTunnel = Self.tunnels.first(where: { $0.identifier == self.identifier }) {
                self.tunnelInfo = savedTunnel

                completionHandler(nil)
            } else {
                completionHandler(NEVPNError(.configurationInvalid))
            }

        }
    }

    func saveToPreferences(completionHandler: ((Error?) -> Void)?) {
        Self.tunnelsLock.withCriticalBlock {
            if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                Self.tunnels[index] = self.tunnelInfo
            } else {
                Self.tunnels.append(self.tunnelInfo)
            }

            completionHandler?(nil)
        }
    }

    func removeFromPreferences(completionHandler: ((Error?) -> Void)?) {
        Self.tunnelsLock.withCriticalBlock {
            if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                Self.tunnels.remove(at: index)
                completionHandler?(nil)
            } else {
                completionHandler?(NEVPNError(.configurationReadWriteFailed))
            }
        }
    }

    static func == (lhs: SimulatorTunnelProviderManager, rhs: SimulatorTunnelProviderManager) -> Bool {
        lhs.identifier == rhs.identifier
    }

}

#endif
