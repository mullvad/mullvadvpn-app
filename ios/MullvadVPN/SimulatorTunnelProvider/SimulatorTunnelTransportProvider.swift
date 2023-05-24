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

final class SimulatorTunnelTransportProvider: RESTTransportProvider {
    private let urlSessionTransport: URLSessionTransport

    init(urlSessionTransport: URLSessionTransport) {
        self.urlSessionTransport = urlSessionTransport
    }

    func transport() -> RESTTransport? {
        urlSessionTransport
    }

    func shadowSocksTransport() -> RESTTransport? {
        urlSessionTransport
    }
}
