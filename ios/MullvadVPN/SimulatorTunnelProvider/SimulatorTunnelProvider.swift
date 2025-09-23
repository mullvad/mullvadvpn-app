//
//  SimulatorTunnelProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 05/02/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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

        private let lock = NSLock()
        nonisolated(unsafe) private var _delegate: SimulatorTunnelProviderDelegate?

        var delegate: SimulatorTunnelProviderDelegate! {
            get {
                lock.lock()
                defer { lock.unlock() }

                return _delegate
            }
            set {
                lock.lock()
                _delegate = newValue
                lock.unlock()
            }
        }

        private init() {}

        func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)? = nil) {
            delegate.handleAppMessage(messageData, completionHandler: completionHandler)
        }
    }

#endif
