//
//  VPNConnectionProtocol.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-09-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

protocol VPNConnectionProtocol: NSObject {
    var status: NEVPNStatus { get }
    var connectedDate: Date? { get }

    func startVPNTunnel() throws
    func startVPNTunnel(options: [String: NSObject]?) throws
    func stopVPNTunnel()
}

protocol VPNTunnelProviderSessionProtocol {
    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws
}
