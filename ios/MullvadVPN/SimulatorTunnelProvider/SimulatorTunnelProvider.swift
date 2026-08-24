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
import NetworkExtension

#if targetEnvironment(simulator)

    class SimulatorTunnelProviderDelegate {
        var connection: SimulatorVPNConnection?

        var protocolConfiguration: NEVPNProtocol {
            connection?.protocolConfiguration ?? NEVPNProtocol()
        }

        var reasserting: Bool {
            get {
                connection?.reasserting ?? false
            }
            set {
                connection?.reasserting = newValue
            }
        }

        func startTunnel(options: [String: NSObject]?, completionHandler: @escaping @Sendable (Error?) -> Void) {
            completionHandler(nil)
        }

        func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping @Sendable () -> Void) {
            completionHandler()
        }

        func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
            completionHandler?(nil)
        }
    }

    final class SimulatorTunnelProvider: Sendable {
        static let shared = SimulatorTunnelProvider()

        nonisolated(unsafe) public var delegate: SimulatorTunnelProviderDelegate!

        private init() {}

        func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)? = nil) {
            delegate.handleAppMessage(messageData, completionHandler: completionHandler)
        }
    }

#endif
