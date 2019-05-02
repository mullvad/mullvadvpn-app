//
//  MullvadAPI.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

private let kMullvadAPIURL = URL(string: "https://api.mullvad.net/rpc/")!

class MullvadAPI {

    class func getRelayList(completion: @escaping (_ result: Result<JsonRpcResponse<RelayList>, Error>) -> Void) -> URLSessionDataTask {
        let urlRequest = try! makeURLRequest(method: "POST", rpcRequest: JsonRpcRequest(method: "relay_list_v2"))

        return URLSession.shared.dataTask(with: urlRequest) { (data, response, error) in
            DispatchQueue.main.async {
                completion(error.flatMap({ Result.failure($0) }) ?? Result(catching: {
                    try self.decodeResponse(data: try data.unwrap())
                }))
            }
        }
    }

    private class func decodeResponse<T: Decodable>(data: Data) throws -> JsonRpcResponse<T> {
        let decoder = defaultJsonDecoder()

        return try decoder.decode(JsonRpcResponse<T>.self, from: data)
    }

    private class func makeURLRequest<T: Encodable>(method: String, rpcRequest: JsonRpcRequest<T>) throws -> URLRequest {
        let encoder = defaultJsonEncoder()

        var urlRequest = URLRequest(url: kMullvadAPIURL)
        urlRequest.httpMethod = method
        urlRequest.httpBody = try encoder.encode(rpcRequest)
        urlRequest.addValue("application/json", forHTTPHeaderField: "Content-Type")

        return urlRequest
    }

}

private func defaultJsonEncoder() -> JSONEncoder {
    let encoder = JSONEncoder()
    encoder.keyEncodingStrategy = .convertToSnakeCase
    encoder.dateEncodingStrategy = .iso8601
    return encoder
}

private func defaultJsonDecoder() -> JSONDecoder {
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    decoder.dateDecodingStrategy = .iso8601
    return decoder
}

