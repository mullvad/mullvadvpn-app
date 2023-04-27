//
//  RESTTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RESTTransport {
    var name: String { get }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable
}

public protocol RESTTransportProvider {
    /// Requests a new transport
    /// - Returns: A transport layer
    func transport() -> RESTTransport?

    /// Requests a Shadowsocks transport
    /// - Returns: A transport layer that proxies the requests to a local Shadowsocks proxy instance locally
    func shadowSocksTransport() -> RESTTransport?
}
