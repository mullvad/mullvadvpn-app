//
//  ShadowsocksConfiguration.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public protocol ShadowsocksLoaderProtocol: Sendable {
    func load() throws -> ShadowsocksConfiguration
    func clear() throws
}

public struct ShadowsocksConfiguration: Codable, Equatable, Sendable {
    public let address: AnyIPAddress
    public let port: UInt16
    public let password: String
    public let cipher: String

    public init(address: AnyIPAddress, port: UInt16, password: String, cipher: String) {
        self.address = address
        self.port = port
        self.password = password
        self.cipher = cipher
    }
}
