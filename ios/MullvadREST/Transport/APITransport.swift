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

    func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse
}

public final class APITransport: APITransportProtocol {
    public var name: String {
        "app-transport"
    }

    public let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse {
        let rustTaskHandle = try requestFactory.makeRequest(request)

        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                guard !Task.isCancelled else {
                    continuation.resume(throwing: CancellationError())
                    return
                }

                rustTaskHandle.start { response in
                    let error: APIError? =
                        if !response.success() {
                            APIError(
                                statusCode: Int(response.statusCode()),
                                errorDescription: response.errorDescription() ?? "",
                                serverResponseCode: response.serverResponseCode()
                            )
                        } else { nil }

                    continuation.resume(
                        returning: ProxyAPIResponse(
                            data: response.body(),
                            error: error,
                            etag: response.etag()
                        )
                    )
                }
            }
        } onCancel: {
            rustTaskHandle.cancel()
        }
    }

    public func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) throws -> Cancellable {
        let apiRequest = try requestFactory.makeRequest(request)

        apiRequest.start { response in
            let error: APIError? =
                if !response.success() {
                    APIError(
                        statusCode: Int(response.statusCode()),
                        errorDescription: response.errorDescription() ?? "",
                        serverResponseCode: response.serverResponseCode()
                    )
                } else { nil }

            completion(
                ProxyAPIResponse(
                    data: response.body(),
                    error: error,
                    etag: response.etag()
                ))
        }
        return apiRequest
    }
}
