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
import TunnelProviderMessaging

final class PacketTunnelTransport: RESTTransport {
    var name: String {
        return "packet-tunnel"
    }

    let tunnelManager: TunnelManager

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) throws -> Cancellable {
        let proxyRequest = try ProxyURLRequest(id: UUID(), urlRequest: request)

        return try tunnelManager.sendRequest(proxyRequest) { result in
            switch result {
            case .cancelled:
                completion(nil, nil, URLError(.cancelled))

            case let .success(reply):
                completion(
                    reply.data,
                    reply.response?.originalResponse,
                    reply.error?.originalError
                )

            case let .failure(error):
                completion(nil, nil, error)
            }
        }
    }
}
