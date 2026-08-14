//
//  PacketTunnelAPITransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import Operations
import PacketTunnelCore

final class PacketTunnelAPITransport: Sendable, APITransportProtocol {
    nonisolated(unsafe) private var cancellable: Cancellable?

    var name: String {
        "packet-tunnel-transport"
    }

    let tunnel: any TunnelProtocol

    init(tunnel: any TunnelProtocol) {
        self.tunnel = tunnel
    }

    public func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse {
        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                guard !Task.isCancelled else {
                    continuation.resume(throwing: TaskError.cancelled)
                    return
                }

                let proxyRequest = ProxyAPIRequest(
                    id: UUID(),
                    request: request
                )

                cancellable = tunnel.sendAPIRequest(proxyRequest) { result in
                    guard !Task.isCancelled else {
                        continuation.resume(throwing: TaskError.cancelled)
                        return
                    }

                    switch result {
                    case let .success(reply):
                        continuation.resume(returning: reply)

                    case let .failure(error):
                        let error = error.isOperationCancellationError ? TaskError.cancelled : error

                        continuation.resume(
                            returning: (ProxyAPIResponse(
                                data: nil,
                                error: APIError(
                                    statusCode: 0,
                                    errorDescription: error.localizedDescription,
                                    serverResponseCode: nil
                                )
                            ))
                        )
                    }
                }
            }
        } onCancel: {
            cancellable?.cancel()
            cancellable = nil
        }
    }

    func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) -> Cancellable {
        let proxyRequest = ProxyAPIRequest(
            id: UUID(),
            request: request
        )

        return tunnel.sendAPIRequest(proxyRequest) { result in
            switch result {
            case let .success(reply):
                completion(reply)

            case let .failure(error):
                let error = error.isOperationCancellationError ? URLError(.cancelled) : error
                completion(
                    ProxyAPIResponse(
                        data: nil,
                        error: APIError(
                            statusCode: 0,
                            errorDescription: error.localizedDescription,
                            serverResponseCode: nil
                        )))
            }
        }
    }
}
