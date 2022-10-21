//
//  TunnelProviderReply.swift
//  TunnelProviderMessaging
//
//  Created by pronebird on 20/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Container type for tunnel provider reply.
public struct TunnelProviderReply<T: Codable>: Codable {
    public var value: T

    public init(_ value: T) {
        self.value = value
    }

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        return try JSONEncoder().encode(self)
    }
}
