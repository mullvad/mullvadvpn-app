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
class AnyTransport: RESTTransport {
    typealias CompletionHandler = (Data?, URLResponse?, Error?) -> Void

    private let handleRequest: () -> AnyResponse

    private let completionLock = NSLock()
    private var completionHandlers: [UUID: CompletionHandler] = [:]

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
        let id = storeCompletion(completionHandler: completion)

        let dispatchWork = DispatchWorkItem {
            let data = (try? response.encode()) ?? Data()
            let httpResponse = HTTPURLResponse(
                url: request.url!,
                statusCode: response.statusCode,
                httpVersion: "1.0",
                headerFields: [:]
            )!
            self.sendCompletion(requestID: id, completion: .success((data, httpResponse)))
        }

        DispatchQueue.global().asyncAfter(deadline: .now() + response.delay, execute: dispatchWork)

        return AnyCancellable {
            dispatchWork.cancel()

            self.sendCompletion(requestID: id, completion: .failure(URLError(.cancelled)))
        }
    }

    private func storeCompletion(completionHandler: @escaping CompletionHandler) -> UUID {
        return completionLock.withLock {
            let id = UUID()
            completionHandlers[id] = completionHandler
            return id
        }
    }

    private func sendCompletion(requestID: UUID, completion: Result<(Data, URLResponse), Error>) {
        let complationHandler = completionLock.withLock {
            return completionHandlers.removeValue(forKey: requestID)
        }
        switch completion {
        case let .success((data, response)):
            complationHandler?(data, response, nil)
        case let .failure(error):
            complationHandler?(nil, nil, error)
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
