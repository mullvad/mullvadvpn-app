//
//  SimulatorTunnelInfo.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-09-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if targetEnvironment(simulator)

import Foundation
import NetworkExtension

final class SimulatorTunnelProviderSession: SimulatorVPNConnection, VPNTunnelProviderSessionProtocol {
    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        SimulatorTunnelProvider.shared.handleAppMessage(
            messageData,
            completionHandler: responseHandler
        )
    }
}

/// A mock struct for tunnel configuration and connection
struct SimulatorTunnelInfo {
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
            connection.protocolConfiguration = protocolConfiguration ?? NEVPNProtocol()
        }
    }

    /// Tunnel description
    var localizedDescription: String?

    /// Designated initializer
    init() {}
}

#endif
