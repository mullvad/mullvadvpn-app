//
//  URLSessionTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension URLSessionTask: Cancellable {}

extension REST {
    public final class URLSessionTransport: RESTTransport {
        public var name: String {
            "url-session"
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

    public final class URLSessionShadowSocksTransport: RESTTransport {
        /// The Shadowsocks proxy instance that proxies all the traffic it receives
        private let shadowSocksProxy: ShadowsocksProxy
        /// The IPv4 representation of the loopback address used by `shadowSocksProxy`
        private let localhost = "127.0.0.1"

        /// The `URLSession` used to send requests via `shadowSocksProxy`
        public let urlSession: URLSession

        public var name: String {
            "shadow-socks-url-session"
        }

        public init(
            urlSession: URLSession,
            shadowSocksConfiguration: ServerShadowsocks,
            shadowSocksBridgeRelay: BridgeRelay,
            addressCache: REST.AddressCache
        ) {
            self.urlSession = urlSession
            let apiAddress = addressCache.getCurrentEndpoint()

            shadowSocksProxy = ShadowsocksProxy(
                forwardAddress: apiAddress.ip,
                forwardPort: apiAddress.port,
                bridgeAddress: shadowSocksBridgeRelay.ipv4AddrIn,
                bridgePort: shadowSocksConfiguration.port,
                password: shadowSocksConfiguration.password,
                cipher: shadowSocksConfiguration.cipher
            )
        }

        public func sendRequest(
            _ request: URLRequest,
            completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
        ) -> Cancellable {
            // Start the Shadowsocks proxy in order to get a local port
            shadowSocksProxy.start()

            // Copy the URL request and rewrite the host and port to point to the Shadowsocks proxy instance
            var urlRequestCopy = request
            urlRequestCopy.url = request.url.flatMap { url in
                var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
                components?.host = localhost
                components?.port = Int(shadowSocksProxy.localPort())
                return components?.url
            }

            let dataTask = urlSession.dataTask(with: urlRequestCopy, completionHandler: completion)
            dataTask.resume()
            return dataTask
        }
    }
}
