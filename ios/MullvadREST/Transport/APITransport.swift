// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
