//
//  AnyTransport.swift
//  MullvadRESTTests
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import MullvadTypes

/// Mock implementation of REST transport that can be used to handle requests without doing any actual networking.
struct AnyTransport: RESTTransport {
    private let handleRequest: () -> AnyResponse

    init(block: @escaping () -> AnyResponse) {
        handleRequest = block
    }

    var name: String {
        return "any-transport"
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        let response = handleRequest()

        let dispatchWork = DispatchWorkItem {
            let data = try! response.encode()
            let httpResponse = HTTPURLResponse(
                url: request.url!,
                statusCode: response.statusCode,
                httpVersion: "1.0",
                headerFields: [:]
            )
            completion(data, httpResponse, nil)
        }

        DispatchQueue.global().asyncAfter(deadline: .now() + response.delay, execute: dispatchWork)

        return AnyCancellable {
            dispatchWork.cancel()
        }
    }
}

struct Response<T: Encodable>: AnyResponse {
    var delay: TimeInterval
    var statusCode: Int
    var value: T

    func encode() throws -> Data {
        return try REST.Coding.makeJSONEncoder().encode(value)
    }
}

protocol AnyResponse {
    var delay: TimeInterval { get }
    var statusCode: Int { get }

    func encode() throws -> Data
}
