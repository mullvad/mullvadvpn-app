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
        let encodableModel = EncodableModel(
            urlRequest: request
        )

        return TunnelManager.shared.sendRequest(message: encodableModel) { result in
            switch result {
            case .cancelled: break
            case let .success(data):
                let decodableModel = try? JSONDecoder().decode(DecodableModel.self, from: data)

                completion(decodableModel?.data, URLResponse(), URLError.badURL as? Error)
            case let .failure(error):
                completion(nil, URLResponse(), error)
            }
        }
    }

    #if DEBUG
    func sendRequest(
        _ httpBody: EncodableModel,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable? {
        return TunnelManager.shared.sendRequest(message: httpBody) { result in
            switch result {
            case .cancelled: break
            case let .success(data):
                let decodableModel = try? JSONDecoder().decode(DecodableModel.self, from: data)

                completion(decodableModel?.data, URLResponse(), URLError.badURL as? Error)
            case let .failure(error):
                completion(nil, URLResponse(), error)
            }
        }
    }
    #endif
}
