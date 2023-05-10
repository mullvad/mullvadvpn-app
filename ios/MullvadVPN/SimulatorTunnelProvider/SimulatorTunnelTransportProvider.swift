//
//  SimulatorTunnelTransportProvider.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-05-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

final class SimulatorTunnelTransportProvider: RESTTransportProvider {
    private let urlSessionTransport: REST.URLSessionTransport

    init(urlSessionTransport: REST.URLSessionTransport) {
        self.urlSessionTransport = urlSessionTransport
    }

    func transport() -> MullvadREST.RESTTransport? {
        urlSessionTransport
    }

    func shadowSocksTransport() -> MullvadREST.RESTTransport? {
        urlSessionTransport
    }
}
