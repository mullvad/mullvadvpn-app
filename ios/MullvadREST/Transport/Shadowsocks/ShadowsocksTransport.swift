//
//  ShadowsocksTransport.swift
//  MullvadTransport
//
//  Created by pronebird on 12/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public final class ShadowsocksTransport: RESTTransport {
    /// The Shadowsocks proxy instance that proxies all the traffic it receives
    private let shadowsocksProxy: ShadowsocksProxy

    /// The IPv4 representation of the loopback address used by `shadowsocksProxy`
    private let localhost = "127.0.0.1"

    /// The `URLSession` used to send requests via `shadowsocksProxy`
    public let urlSession: URLSession

    public var name: String {
        "shadow-socks-url-session"
    }

    public init(
        urlSession: URLSession,
        configuration: ShadowsocksConfiguration,
        addressCache: REST.AddressCache
    ) {
        self.urlSession = urlSession
        let apiAddress = addressCache.getCurrentEndpoint()

        shadowsocksProxy = ShadowsocksProxy(
            forwardAddress: apiAddress.ip,
            forwardPort: apiAddress.port,
            bridgeAddress: configuration.address,
            bridgePort: configuration.port,
            password: configuration.password,
            cipher: configuration.cipher
        )
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
    ) -> Cancellable {
        // Start the Shadowsocks proxy in order to get a local port
        shadowsocksProxy.start()

        // Copy the URL request and rewrite the host and port to point to the Shadowsocks proxy instance
        var urlRequestCopy = request
        urlRequestCopy.url = request.url.flatMap { url in
            var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
            components?.host = localhost
            components?.port = Int(shadowsocksProxy.localPort())
            return components?.url
        }

        let dataTask = urlSession.dataTask(with: urlRequestCopy, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
