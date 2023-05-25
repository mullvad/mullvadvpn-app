//
//  URLSessionTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

extension URLSessionTask: Cancellable {}

public final class URLSessionTransport: RESTTransport {
    public var name: String {
        return "url-session"
    }

    public let urlSession: URLSession
    public init(urlSession: URLSession) {
        self.urlSession = urlSession
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
    ) -> Cancellable {
        let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}

public final class URLSessionShadowsocksTransport: RESTTransport {
    public var name: String {
        return "shadow-socks-url-session"
    }

    public let urlSession: URLSession
    private let shadowsocksProxy: ShadowsocksProxy

    public init(
        urlSession: URLSession,
        shadowsocksConfiguration: REST.ServerShadowsocks,
        shadowsocksBridgeRelay: REST.BridgeRelay
    ) {
        self.urlSession = urlSession
        shadowsocksProxy = ShadowsocksProxy(
            remoteAddress: shadowsocksBridgeRelay.ipv4AddrIn,
            remotePort: shadowsocksConfiguration.port,
            password: shadowsocksConfiguration.password,
            cipher: shadowsocksConfiguration.cipher
        )
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
    ) -> Cancellable {
        // Start the shadow socks proxy in order to get a local port
        shadowsocksProxy.start()

        // Copy the URL request and rewrite the host and port to point to the shadow socks proxy instance
        var urlRequestCopy = request
        urlRequestCopy.url = request.url.flatMap { url in
            var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
            components?.host = "127.0.0.1"
            components?.port = Int(shadowsocksProxy.localPort())
            return components?.url
        }

        let dataTask = urlSession.dataTask(with: urlRequestCopy, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
