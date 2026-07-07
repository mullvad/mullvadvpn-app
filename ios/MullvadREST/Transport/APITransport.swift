//
//  APITransport.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes

public protocol APITransportProtocol {
    var name: String { get }

    func sendRequest(_ request: APIRequest, completion: @escaping @Sendable (ProxyAPIResponse) -> Void) throws
        -> Cancellable
}
extension APITransportProtocol {
    func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse {
        try await withCheckedThrowingContinuation { continuation in
            do {
                _ = try sendRequest(request) { response in
                    if let error = response.error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(returning: response)
                    }
                }
            } catch {
                continuation.resume(throwing: error)
            }
        }
    }
}

public final class APITransport: APITransportProtocol {
    public var name: String {
        "app-transport"
    }

    public let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) throws -> Cancellable {
        let apiRequest = requestFactory.makeRequest(request)

        return try apiRequest { response in
            let error: APIError? =
                if !response.success {
                    APIError(
                        statusCode: Int(response.statusCode),
                        errorDescription: response.errorDescription ?? "",
                        serverResponseCode: response.serverResponseCode
                    )
                } else { nil }

            completion(
                ProxyAPIResponse(
                    data: response.body,
                    error: error,
                    etag: response.etag
                ))
        }
    }
}
