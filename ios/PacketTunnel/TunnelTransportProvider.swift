//
//  TunnelTransportProvider.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2023-04-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import RelayCache

final class TunnelTransportProvider: RESTTransportProvider {
    private let urlSessionTransport: REST.URLSessionTransport
    private let relayCache: RelayCache

    init(urlSessionTransport: REST.URLSessionTransport, relayCache: RelayCache) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
    }

    func transport() -> MullvadREST.RESTTransport? {
        urlSessionTransport
    }

    func selectNextTransport() {}
}
