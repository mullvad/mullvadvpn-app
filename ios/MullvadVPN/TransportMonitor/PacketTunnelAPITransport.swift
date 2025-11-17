//
//  PacketTunnelAPITransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import Operations
import PacketTunnelCore

final class PacketTunnelAPITransport: APITransportProtocol {
    var name: String {
        "packet-tunnel-transport"
    }

    let tunnel: any TunnelProtocol

    init(tunnel: any TunnelProtocol) {
        self.tunnel = tunnel
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
