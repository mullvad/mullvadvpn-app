//
//  TunnelIPCCoding.swift
//  TunnelIPCCoding
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TunnelIPC {
    /// An extension used for encoding and decoding Tunnel IPC messages.
    enum Coding {}
}

extension TunnelIPC.Coding {
    static func encodeRequest(_ message: TunnelIPC.Request) throws -> Data {
        return try JSONEncoder().encode(message)
    }

    static func decodeRequest(_ messageData: Data) throws -> TunnelIPC.Request {
        return try JSONDecoder().decode(TunnelIPC.Request.self, from: messageData)
    }

    static func encodeResponse<T>(_ response: T) throws -> Data where T: Codable {
        return try JSONEncoder().encode(TunnelIPC.Response(value: response))
    }

    static func decodeResponse<T>(_ type: T.Type, from data: Data) throws -> T where T: Codable {
        return try JSONDecoder().decode(TunnelIPC.Response<T>.self, from: data).value
    }
}
