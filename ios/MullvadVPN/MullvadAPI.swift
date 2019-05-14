//
//  MullvadAPI.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit

private let kMullvadAPIURL = URL(string: "https://api.mullvad.net/rpc/")!

class MullvadAPI {

    class func getRelayList() -> JSONRequestProcedure<Void, JsonRpcResponse<RelayList>> {
        return JSONRequestProcedure(requestBuilder: {
            try makeURLRequest(method: "POST", rpcRequest: JsonRpcRequest(method: "relay_list_v2"))
        })
    }

    class func getAccountData(accountToken: String? = nil) -> JSONRequestProcedure<String, JsonRpcResponse<AccountData>> {
        return JSONRequestProcedure(input: accountToken, requestBuilder: {
            try makeURLRequest(
                method: "POST",
                rpcRequest: JsonRpcRequest(method: "get_account_data", params: [$0])
            )
        })
    }

    private class func makeURLRequest<T: Encodable>(method: String, rpcRequest: JsonRpcRequest<T>) throws -> URLRequest {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601

        var urlRequest = URLRequest(url: kMullvadAPIURL)
        urlRequest.httpMethod = method
        urlRequest.httpBody = try encoder.encode(rpcRequest)
        urlRequest.addValue("application/json", forHTTPHeaderField: "Content-Type")

        return urlRequest
    }

}
