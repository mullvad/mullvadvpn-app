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
    import NetworkExtension

    final class SimulatorTunnelProviderManager: NSObject, VPNTunnelProviderManagerProtocol {
        static let tunnelsLock = NSRecursiveLock()
        nonisolated(unsafe) fileprivate static var tunnels = [SimulatorTunnelInfo]()

        private let lock = NSLock()
        private var tunnelInfo: SimulatorTunnelInfo
        private var identifier: String {
            lock.withLock {
                tunnelInfo.identifier
            }
        }

        var isOnDemandEnabled: Bool {
            get {
                lock.withLock {
                    tunnelInfo.isOnDemandEnabled
                }
            }
            set {
                lock.withLock {
                    tunnelInfo.isOnDemandEnabled = newValue
                }
            }
        }

        var onDemandRules: [NEOnDemandRule] {
            get {
                lock.withLock {
                    tunnelInfo.onDemandRules
                }
            }
            set {
                lock.withLock {
                    tunnelInfo.onDemandRules = newValue
                }
            }
        }

        var isEnabled: Bool {
            get {
                lock.withLock {
                    tunnelInfo.isEnabled
                }
            }
            set {
                lock.withLock {
                    tunnelInfo.isEnabled = newValue
                }
            }
        }

        var protocolConfiguration: NEVPNProtocol? {
            get {
                lock.withLock {
                    tunnelInfo.protocolConfiguration
                }
            }
            set {
                lock.withLock {
                    tunnelInfo.protocolConfiguration = newValue
                }
            }
        }

        var localizedDescription: String? {
            get {
                lock.withLock {
                    return tunnelInfo.localizedDescription
                }
            }
            set {
                lock.withLock {
                    tunnelInfo.localizedDescription = newValue
                }
            }
        }

        var connection: SimulatorVPNConnection {
            lock.withLock {
                tunnelInfo.connection
            }
        }

        static func loadAllFromPreferences(
            completionHandler: (
                [SimulatorTunnelProviderManager]?,
                Error?
            ) -> Void
        ) {
            let tunnelProviders = Self.tunnelsLock.withLock {
                tunnels.map { tunnelInfo in
                    SimulatorTunnelProviderManager(tunnelInfo: tunnelInfo)
                }
            }
            completionHandler(tunnelProviders, nil)
        }

        static func loadAllFromPreferences() async throws -> [SimulatorTunnelProviderManager] {
            tunnels.map { tunnelInfo in
                SimulatorTunnelProviderManager(tunnelInfo: tunnelInfo)
            }

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

            Self.tunnelsLock.withLock {
                if let savedTunnel = Self.tunnels.first(where: { $0.identifier == self.identifier }) {
                    tunnelInfo = savedTunnel
                } else {
                    error = NEVPNError(.configurationInvalid)
                }
            }

            completionHandler(error)
        }

        func saveToPreferences(completionHandler: ((Error?) -> Void)?) {
            Self.tunnelsLock.withLock {
                if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                    Self.tunnels[index] = tunnelInfo
                } else {
                    Self.tunnels.append(tunnelInfo)
                }

            }
            completionHandler?(nil)
        }

        func removeFromPreferences(completionHandler: ((Error?) -> Void)?) {
            var error: NEVPNError?

            Self.tunnelsLock.withLock {
                if let index = Self.tunnels.firstIndex(where: { $0.identifier == self.identifier }) {
                    Self.tunnels.remove(at: index)
                } else {
                    error = NEVPNError(.configurationReadWriteFailed)
                }
            }

            completionHandler?(error)
        }

        override func isEqual(_ object: Any?) -> Bool {
            guard let other = object as? Self else { return false }
            return self.identifier == other.identifier
        }
    }

#endif
