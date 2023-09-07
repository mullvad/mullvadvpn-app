//
//  SimulatorVPNConnection.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-09-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import NetworkExtension

class SimulatorVPNConnection: NSObject, VPNConnectionProtocol {
    // Protocol configuration is automatically synced by `SimulatorTunnelInfo`
    var protocolConfiguration = NEVPNProtocol()

    private let lock = NSRecursiveLock()
    private var _status: NEVPNStatus = .disconnected
    private var _reasserting = false
    private var _connectedDate: Date?

    private(set) var status: NEVPNStatus {
        get {
            lock.lock()
            defer { lock.unlock() }

            return _status
        }
        set {
            lock.lock()

            if _status != newValue {
                _status = newValue

                // Send notification while holding the lock. This should enable the receiver
                // to fetch the `SimulatorVPNConnection.status` before the concurrent code gets
                // opportunity to change it again.
                postStatusDidChangeNotification()
            }

            lock.unlock()
        }
    }

    var reasserting: Bool {
        get {
            lock.lock()
            defer { lock.unlock() }

            return _reasserting
        }
        set {
            lock.lock()

            if _reasserting != newValue {
                _reasserting = newValue

                if newValue {
                    status = .reasserting
                } else {
                    status = .connected
                }
            }

            lock.unlock()
        }
    }

    private(set) var connectedDate: Date? {
        get {
            lock.lock()
            defer { lock.unlock() }

            return _connectedDate
        }
        set {
            lock.lock()
            _connectedDate = newValue
            lock.unlock()
        }
    }

    func startVPNTunnel() throws {
        try startVPNTunnel(options: nil)
    }

    func startVPNTunnel(options: [String: NSObject]?) throws {
        SimulatorTunnelProvider.shared.delegate.connection = self

        status = .connecting

        SimulatorTunnelProvider.shared.delegate.startTunnel(options: options) { error in
            if error == nil {
                self.status = .connected
                self.connectedDate = Date()
            } else {
                self.status = .disconnected
                self.connectedDate = nil
            }
        }
    }

    func stopVPNTunnel() {
        status = .disconnecting

        SimulatorTunnelProvider.shared.delegate.stopTunnel(with: .userInitiated) {
            self.status = .disconnected
            self.connectedDate = nil
        }
    }

    private func postStatusDidChangeNotification() {
        NotificationCenter.default.post(name: .NEVPNStatusDidChange, object: self)
    }
}

#endif
