//
//  MullvadAPI.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network
import ProcedureKit

private let kMullvadAPIURL = URL(string: "https://api.mullvad.net/rpc/")!

class MullvadAPI {

    struct WireguardKeyRequest: Codable {
        var accountToken: String
        var publicKey: Data
    }

    class func getRelayList() -> JsonRequestProcedure<Void, JsonRpcResponse<RelayList>> {
        return JsonRequestProcedure(requestBuilder: {
            try makeURLRequest(method: "POST",
                               rpcRequest: JsonRpcRequest(method: "relay_list_v2", params: []))
        })
    }

    class func getAccountExpiry(accountToken: String? = nil) -> JsonRequestProcedure<String, JsonRpcResponse<Date>> {
        return JsonRequestProcedure(input: accountToken, requestBuilder: {
            try makeURLRequest(
                method: "POST",
                rpcRequest: JsonRpcRequest(method: "get_expiry", params: [AnyEncodable($0)])
            )
        })
    }

    class func verifyAccountToken(_ accountToken: String? = nil) -> AccountVerificationProcedure {
        return AccountVerificationProcedure(accountToken: accountToken)
    }

    class func pushWireguardKey(_ pushRequest: WireguardKeyRequest? = nil) -> JsonRequestProcedure<WireguardKeyRequest, JsonRpcResponse<WireguardAssociatedAddresses>> {
        return JsonRequestProcedure(input: pushRequest, requestBuilder: { (input) -> URLRequest in
            let rpcRequest = JsonRpcRequest(method: "push_wg_key", params: [
                AnyEncodable(input.accountToken),
                AnyEncodable(input.publicKey)
                ])
            return try makeURLRequest(method: "POST", rpcRequest: rpcRequest)
        })
    }

    class func checkWireguardKey(_ pushRequest: WireguardKeyRequest? = nil) -> JsonRequestProcedure<WireguardKeyRequest, JsonRpcResponse<Bool>> {
        return JsonRequestProcedure(input: pushRequest, requestBuilder: { (input) -> URLRequest in
            let rpcRequest = JsonRpcRequest(method: "check_wg_key", params: [
                AnyEncodable(input.accountToken),
                AnyEncodable(input.publicKey)
                ])
            return try makeURLRequest(method: "POST", rpcRequest: rpcRequest)
        })
    }

    private class func makeURLRequest(method: String, rpcRequest: JsonRpcRequest) throws -> URLRequest {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601
        encoder.dataEncodingStrategy = .base64

        var urlRequest = URLRequest(url: kMullvadAPIURL)
        urlRequest.httpMethod = method
        urlRequest.httpBody = try encoder.encode(rpcRequest)
        urlRequest.addValue("application/json", forHTTPHeaderField: "Content-Type")

        return urlRequest
    }

}
