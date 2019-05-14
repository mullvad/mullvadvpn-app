//
//  JsonRpc.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

typealias JsonRpcRequestId = String
struct JsonRpcRequest<T: Encodable>: Encodable {
    let version = "2.0"
    let id: JsonRpcRequestId = UUID().uuidString
    let method: String
    let params: [T]

    fileprivate enum CodingKeys: String, CodingKey {
        case version = "jsonrpc", id, method, params
    }
}

extension JsonRpcRequest where T == NoData {
    init(method: String) {
        self.init(method: method, params: [])
    }
}

struct NoData: Encodable {}

class JsonRpcResponseError: Error, Decodable {
    let serverErrorMessage: String

    init(serverErrorMessage: String) {
        self.serverErrorMessage = serverErrorMessage
    }

    var localizedDescription: String? {
        return serverErrorMessage
    }

    required init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        serverErrorMessage = try container.decode(String.self)
    }
}

struct JsonRpcResponse<T: Decodable>: Decodable {
    let version: String
    let id: JsonRpcRequestId
    let result: Result<T, JsonRpcResponseError>

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
            self.result = .failure(try container.decode(JsonRpcResponseError.self, forKey: .error))
        }
    }
}
