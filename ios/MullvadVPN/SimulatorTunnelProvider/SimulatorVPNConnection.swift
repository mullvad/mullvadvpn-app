// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

#if targetEnvironment(simulator)

    import Foundation
    import MullvadREST
    import NetworkExtension

    class SimulatorVPNConnection: NSObject, VPNConnectionProtocol, @unchecked Sendable {
        // Protocol configuration is automatically synced by `SimulatorTunnelInfo`
        var protocolConfiguration = NEVPNProtocol()

        private let lock = NSRecursiveLock()
        private var _status: NEVPNStatus = .disconnected
        private var _reasserting = false
        private var _connectedDate: Date?

        private(set) var status: NEVPNStatus {
            get {
                lock.withLock {
                    _status
                }
            }
            set {
                lock.withLock {
                    if _status != newValue {
                        _status = newValue

                        // Send notification while holding the lock. This should enable the receiver
                        // to fetch the `SimulatorVPNConnection.status` before the concurrent code gets
                        // opportunity to change it again.
                        postStatusDidChangeNotification()
                    }
                }
            }
        }

        var reasserting: Bool {
            get {
                lock.withLock {
                    _reasserting
                }
            }
            set {
                lock.withLock {
                    if _reasserting != newValue {
                        _reasserting = newValue

                        if newValue {
                            status = .reasserting
                        } else {
                            status = .connected
                        }
                    }
                }
            }
        }

        private(set) var connectedDate: Date? {
            get {
                lock.withLock {
                    _connectedDate
                }
            }
            set {
                lock.withLock {
                    _connectedDate = newValue
                }
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
                } else if error is NoRelaysSatisfyingConstraintsError {
                    self.reasserting = true
                    self.connectedDate = nil
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
