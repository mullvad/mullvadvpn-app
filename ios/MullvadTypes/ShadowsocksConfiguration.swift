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
