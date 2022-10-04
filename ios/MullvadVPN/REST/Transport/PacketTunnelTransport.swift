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
    ) -> Cancellable? {
        let encodableModel = TransportMessage(
            urlRequest: request
        )

        return TunnelManager.shared.sendRequest(message: encodableModel) { result in
            switch result {
            case .cancelled: break
            case let .success(reply):
                completion(reply.data,
                           reply.response?.originalResponse(),
                           reply.error?.originalError())
            case let .failure(error):
                completion(nil,
                           nil,
                           error)
            }
        }
    }

    #if DEBUG
    func sendRequest(
        _ httpBody: TransportMessage,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable? {
        return TunnelManager.shared.sendRequest(message: httpBody) { result in
            switch result {
            case .cancelled: break
            case let .success(reply):
                completion(reply.data,
                           reply.response?.originalResponse(),
                           reply.error?.originalError())
            case let .failure(error):
                completion(nil,
                           nil,
                           error)
            }
        }
    }
    #endif
}
