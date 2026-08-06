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
    private var rustTaskHandle: MullvadApiCancellable?

    public var name: String {
        "app-transport"
    }

    public let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse {
        // We want to create the task here, eg:
        // let rustTaskHandle = create_the_task()

        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                guard !Task.isCancelled else {
                    continuation.resume(throwing: TaskError.cancelled)
                    return
                }

                let apiRequest = requestFactory.makeRequest(request)

                do {
                    // Then we want to start the task here, eg:
                    // rustTaskHandle.doTheThing { response }

                    rustTaskHandle = try apiRequest { response in
                        let error: APIError? =
                            if !response.success {
                                APIError(
                                    statusCode: Int(response.statusCode),
                                    errorDescription: response.errorDescription ?? "",
                                    serverResponseCode: response.serverResponseCode
                                )
                            } else { nil }

                        continuation.resume(
                            returning: ProxyAPIResponse(
                                data: response.body,
                                error: error,
                                etag: response.etag
                            )
                        )
                    }
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        } onCancel: {
            // And finally be able to cancel it here.
            // self?.rustTaskHandle?.cancel()
            // self?.rustTaskHandle = nil
        }
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
