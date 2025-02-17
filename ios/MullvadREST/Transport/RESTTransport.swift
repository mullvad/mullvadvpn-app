//
//  RESTTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime
import MullvadTypes
import Operations

public enum APIRequest: Codable {
    case getAddressList(retryStrategy: REST.RetryStrategy)

    var retryStrategy: REST.RetryStrategy {
        switch self {
        case let .getAddressList(retryStrategy: strategy):
            return strategy
        }
    }
}

public struct ProxyAPIRequest: Codable {
    public let id: UUID
    public let request: APIRequest

    public init(id: UUID, request: APIRequest) {
        self.id = id
        self.request = request
    }
}

public struct ProxyAPIResponse: Codable, Sendable {
    public let data: Data?
    public let error: APIErrorWrapper?

    public init(data: Data?, error: APIErrorWrapper?) {
        self.data = data
        self.error = error
    }
}

public struct APIErrorWrapper: Codable, Sendable {
    public let code: Int?
    public let localizedDescription: String

    public init?(_ error: Error) {
        localizedDescription = error.localizedDescription
        code = (error as? URLError)?.errorCode
    }

    public var originalError: Error? {
        guard let code else { return nil }
        return URLError(URLError.Code(rawValue: code))
    }
}

public protocol RESTTransport: Sendable {
    var name: String { get }

    func sendRequest(_ request: URLRequest, completion: @escaping @Sendable (Data?, URLResponse?, Error?) -> Void)
        -> Cancellable
}

public protocol APITransportProtocol {
    var name: String { get }

    func sendRequest(_ request: APIRequest, completion: @escaping @Sendable (ProxyAPIResponse) -> Void)
        -> Cancellable
}

public final class APITransport: APITransportProtocol {
    public var name: String {
        "app"
    }

    public let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) -> Cancellable {
        let apiRequest = requestFactory.makeRequest(request)

        return apiRequest { response in
            let response = ProxyAPIResponse(
                data: response.body,
                error: APIErrorWrapper(
                    NSError(
                        domain: response.serverResponseCode ?? "",
                        code: Int(response.statusCode)
                    )
                )
            )
            completion(response)
        }
    }
}
