//
//  JsonRpc.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Encodable {
    fileprivate func encode(to container: inout SingleValueEncodingContainer) throws {
        try container.encode(self)
    }
}

struct AnyEncodable: Encodable {
    let value: Encodable

    init(_ value: Encodable) {
        self.value = value
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try value.encode(to: &container)
    }
}

struct JsonRpcRequest: Encodable {
    let version = "2.0"
    let id = UUID().uuidString
    let method: String
    let params: [AnyEncodable]

    fileprivate enum CodingKeys: String, CodingKey {
        case version = "jsonrpc", id, method, params
    }
}

class JsonRpcResponseError<ResponseCode>: Error, Decodable
    where ResponseCode: Decodable
{
    let code: ResponseCode
    let message: String

    var localizedDescription: String {
        return message
    }

    private enum CodingKeys: String, CodingKey {
        case code, message
    }

    required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        code = try container.decode(ResponseCode.self, forKey: .code)
        message = try container.decode(String.self, forKey: .message)
    }
}

struct JsonRpcResponse<T, ResponseCode>: Decodable
    where
    T: Decodable, ResponseCode: Decodable
{
    let version: String
    let id: String
    let result: Result<T, JsonRpcResponseError<ResponseCode>>

    private enum CodingKeys: String, CodingKey {
        case version = "jsonrpc", id, result, error
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        self.version = try container.decode(String.self, forKey: .version)
        self.id = try container.decode(String.self, forKey: .id)

        if container.contains(.result) {
            self.result = .success(try container.decode(T.self, forKey: .result))
        } else {
            self.result = .failure(try container.decode(JsonRpcResponseError<ResponseCode>.self, forKey: .error))
        }
    }
}
