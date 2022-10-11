//
//  PacketTunnelTransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class PacketTunnelTransport: RESTTransport {
    var name: String {
        "packet-tunnel"
    }

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
            case .cancelled:
                completion(nil, nil, URLError(.cancelled))
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

    func isTimeoutError(_ error: Error) -> Bool {
        if case .timeout = error as? SendTunnelProviderMessageError {
            return true
        } else if let error = error as? URLError, error.code == .timedOut {
            return true
        }
        return false
    }
}
