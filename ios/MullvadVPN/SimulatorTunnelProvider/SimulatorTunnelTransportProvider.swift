//
//  SimulatorTunnelTransportProvider.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-05-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTransport
import MullvadTypes

final class SimulatorTunnelTransportProvider: RESTTransport {
    private let urlSessionTransport: URLSessionTransport

    init(urlSessionTransport: URLSessionTransport) {
        self.urlSessionTransport = urlSessionTransport
    }

    var name: String {
        "SimulatorTunnelTransportProvider"
    }

    func sendRequest(_ request: URLRequest, completion: @escaping (Data?, URLResponse?, Error?) -> Void) -> MullvadTypes
        .Cancellable
    {
        urlSessionTransport.sendRequest(request, completion: completion)
    }
}
