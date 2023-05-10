//
//  PacketTunnelTransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol MullvadREST.RESTTransport
import MullvadTypes
import Operations
import TunnelProviderMessaging

struct PacketTunnelTransport: RESTTransport {
    var name: String {
        return "packet-tunnel"
    }

    let tunnel: Tunnel
    let useShadowsocksTransport: Bool

    init(tunnel: Tunnel, useShadowsocksTransport: Bool) {
        self.tunnel = tunnel
        self.useShadowsocksTransport = useShadowsocksTransport
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        let proxyRequest = ProxyURLRequest(
            id: UUID(),
            urlRequest: request,
            useShadowsocksTransport: useShadowsocksTransport
        )

        // If the URL provided to the proxy request was invalid, indicate failure via `.badURL` and return a no-op.
        guard let proxyRequest = proxyRequest else {
            completion(nil, nil, URLError(.badURL))
            return AnyCancellable {}
        }

        return tunnel.sendRequest(proxyRequest) { result in
            switch result {
            case let .success(reply):
                completion(
                    reply.data,
                    reply.response?.originalResponse,
                    reply.error?.originalError
                )

            case let .failure(error):
                let returnError = error.isOperationCancellationError ? URLError(.cancelled) : error

                completion(nil, nil, returnError)
            }
        }
    }
}
