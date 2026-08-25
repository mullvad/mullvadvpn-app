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

public protocol APIRequestProxyProtocol {
    func sendRequest(_ proxyRequest: ProxyAPIRequest) async throws -> ProxyAPIResponse
    func cancelRequest(identifier: UUID)
}

/// Network request proxy capable of passing serializable requests and responses over the given transport provider.
public final class APIRequestProxy: APIRequestProxyProtocol, @unchecked Sendable {

    private let transportProvider: APITransportProviderProtocol
    private let dispatchQueue: DispatchQueue

    private var proxiedRequests: [UUID: RequestTask] = [:]

    public init(
        dispatchQueue: DispatchQueue,
        transportProvider: APITransportProviderProtocol
    ) {
        self.dispatchQueue = dispatchQueue
        self.transportProvider = transportProvider
    }

    public func sendRequest(
        _ proxyRequest: ProxyAPIRequest
    ) async throws -> ProxyAPIResponse {
        guard let transport = transportProvider.makeTransport() else {
            cancelRequest(identifier: proxyRequest.id)

            return ProxyAPIResponse(
                data: nil,
                error: APIError(
                    statusCode: 0,
                    errorDescription: REST.InternalTransportError.noTransport.errorDescription,
                    serverResponseCode: nil
                )
            )
        }

        let requestTask = RequestTask(
            task: Task {
                try await transport.sendRequest(
                    proxyRequest.request
                )
            }
        )

        let oldTask = replaceRequest(
            identifier: proxyRequest.id,
            task: requestTask
        )

        oldTask?.task.cancel()

        defer {
            removeRequest(
                identifier: proxyRequest.id,
                task: requestTask
            )
        }

        do {
            return try await requestTask.task.value
        } catch is CancellationError {
            throw CancellationError()
        } catch {
            throw error
        }
    }

    public func cancelRequest(identifier: UUID) {
        let task = dispatchQueue.sync {
            proxiedRequests.removeValue(forKey: identifier)
        }

        task?.task.cancel()
    }

    private func replaceRequest(
        identifier: UUID,
        task: RequestTask
    ) -> RequestTask? {
        dispatchQueue.sync {
            proxiedRequests.updateValue(
                task,
                forKey: identifier
            )
        }
    }

    private func removeRequest(
        identifier: UUID,
        task: RequestTask
    ) {
        dispatchQueue.sync {
            guard proxiedRequests[identifier] === task else {
                return
            }

            proxiedRequests.removeValue(forKey: identifier)
        }
    }
}

private final class RequestTask: @unchecked Sendable {

    let task: Task<ProxyAPIResponse, Error>

    init(task: Task<ProxyAPIResponse, Error>) {
        self.task = task
    }
}
