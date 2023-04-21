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
            return "url-session"
        }

        public let urlSession: URLSession
        private let shadowSocksConfiguration: ServerShadowsocks
        private let shadowSocksBridgeRelay: BridgeRelay
        
        private var shadowSocksProxy: ShadowSocksProxy!

        public init(urlSession: URLSession,
                    shadowSocksConfiguration: ServerShadowsocks,
                    shadowSocksBridgeRelay: BridgeRelay) {
            self.urlSession = urlSession
            self.shadowSocksConfiguration = shadowSocksConfiguration
            self.shadowSocksBridgeRelay = shadowSocksBridgeRelay
        }

        deinit {
            shadowSocksProxy.stop()
        }
        
        public func sendRequest(
            _ request: URLRequest,
            completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
        ) throws -> Cancellable {

            // Start the shadowSocks proxy
            
            shadowSocksProxy = ShadowSocksProxy(remoteAddress: shadowSocksBridgeRelay.ipv4AddrIn,
                             remotePort: shadowSocksConfiguration.port,
                             password: shadowSocksConfiguration.password,
                             cipher: shadowSocksConfiguration.cipher)
            
            shadowSocksProxy.start()
            
            let urlCopy = request.url
            let originalRequest = request as NSURLRequest
            let urlRequestCopy = originalRequest.mutableCopy() as! NSMutableURLRequest

            var components = URLComponents(string: urlCopy!.absoluteString)!
            components.host = "127.0.0.1"
            components.port = Int(shadowSocksProxy.localPort())
            
            urlRequestCopy.url = components.url
            let rewrittenURLRequest = urlRequestCopy as URLRequest
            
            let dataTask = urlSession.dataTask(with: rewrittenURLRequest, completionHandler: completion)
            dataTask.resume()
            return dataTask
        }
    }
}
