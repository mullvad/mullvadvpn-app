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
            return "url-session"
        }

        public let urlSession: URLSession

        public init(urlSession: URLSession) {
            self.urlSession = urlSession
        }

        public func sendRequest(
            _ request: URLRequest,
            completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
        ) throws -> Cancellable {
            let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
            dataTask.resume()
            return dataTask
        }
    }

    public final class URLSessionShadowSocksTransport: RESTTransport {
        public var name: String {
            return "shadow-socks-url-session"
        }

        public let urlSession: URLSession
        private let shadowSocksProxy: ShadowSocksProxy

        public init(
            urlSession: URLSession,
            shadowSocksConfiguration: ServerShadowsocks,
            shadowSocksBridgeRelay: BridgeRelay
        ) {
            self.urlSession = urlSession
            self.shadowSocksProxy = ShadowSocksProxy(
                remoteAddress: shadowSocksBridgeRelay.ipv4AddrIn,
                remotePort: shadowSocksConfiguration.port,
                password: shadowSocksConfiguration.password,
                cipher: shadowSocksConfiguration.cipher
            )

        }

        deinit {
            shadowSocksProxy.stop()
        }

        public func sendRequest(
            _ request: URLRequest,
            completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
        ) throws -> Cancellable {
            // Start the shadow socks proxy in order to get a local port
            shadowSocksProxy.start()

            // Copy the URL request and rewrite the host and port to point to the shadow socks proxy instance
            var urlRequestCopy = request
            urlRequestCopy.url = request.url.flatMap { url in
                var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
                components?.host = "127.0.0.1"
                components?.port = Int(shadowSocksProxy.localPort())
                return components?.url
            }

            let dataTask = urlSession.dataTask(with: urlRequestCopy, completionHandler: completion)
            dataTask.resume()
            return dataTask
        }
    }
}
