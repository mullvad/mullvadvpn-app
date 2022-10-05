//
//  PacketTunnelTransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct PacketTunnelTransport: RESTTransport {
    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) throws -> Cancellable {
        let message = try TransportMessage(
            /// Create unique request UUID and store it along the URLSessionTask in a dictionary.
            id: UUID(),
            urlRequest: request
        )

        return try TunnelManager.shared.sendRequest(message: message) { result in
            switch result {
            case .cancelled: break
            case let .success(reply):
                completion(
                    reply.data,
                    reply.response?.originalResponse(),
                    reply.error?.originalError()
                )
            case let .failure(error):
                completion(
                    nil,
                    nil,
                    error
                )
            }
        }
    }
}
