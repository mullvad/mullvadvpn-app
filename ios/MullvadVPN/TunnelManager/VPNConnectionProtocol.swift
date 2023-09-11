//
//  VPNConnectionProtocol.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-09-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

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

extension NEVPNConnection: VPNConnectionProtocol {}
extension NETunnelProviderSession: VPNTunnelProviderSessionProtocol {}
extension NETunnelProviderManager: VPNTunnelProviderManagerProtocol {}
