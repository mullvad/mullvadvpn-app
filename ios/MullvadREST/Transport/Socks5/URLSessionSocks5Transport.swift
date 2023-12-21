//
//  URLSessionSocks5Transport.swift
//  MullvadTransport
//
//  Created by pronebird on 23/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes

/// Transport that passes URL requests over the local socks forwarding proxy.
public class URLSessionSocks5Transport: RESTTransport {
    /// Socks5 forwarding proxy.
    private let socksProxy: Socks5ForwardingProxy

    /// The IPv4 representation of the loopback address used by `socksProxy`.
    private let localhost = "127.0.0.1"

    /// The `URLSession` used to send requests via `socksProxy`.
    public let urlSession: URLSession

    public var name: String {
        "socks5-url-session"
    }

    private let logger = Logger(label: "URLSessionSocks5Transport")

    /**
     Instantiates new socks5 transport.

     - Parameters:
         - urlSession: an instance of URLSession used for sending requests.
         - configuration: SOCKS5 configuration
         - addressCache: an address cache
     */
    public init(
        urlSession: URLSession,
        configuration: Socks5Configuration,
        addressCache: REST.AddressCache
    ) {
        self.urlSession = urlSession

        let apiAddress = addressCache.getCurrentEndpoint()

        socksProxy = Socks5ForwardingProxy(
            socksProxyEndpoint: configuration.nwEndpoint,
            remoteServerEndpoint: apiAddress.socksEndpoint
        )

        socksProxy.setErrorHandler { [weak self] error in
            self?.logger.error(error: error, message: "Socks proxy failed at runtime.")
        }
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        // Listen port should be set when socks proxy is ready. Otherwise start proxy and only then start the data task.
        if let localPort = socksProxy.listenPort {
            return startDataTask(request: request, localPort: localPort, completion: completion)
        } else {
            return sendDeferred(request: request, completion: completion)
        }
    }

    /// Starts socks proxy then executes the data task.
    private func sendDeferred(
        request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        let chain = CancellableChain()

        socksProxy.start { [weak self, weak socksProxy] error in
            if let error {
                completion(nil, nil, error)
            } else if let self, let localPort = socksProxy?.listenPort {
                let token = self.startDataTask(request: request, localPort: localPort, completion: completion)

                // Propagate cancellation from the chain to the data task cancellation token.
                chain.link(token)
            } else {
                completion(nil, nil, URLError(.cancelled))
            }
        }

        return chain
    }

    /// Execute data task, rewriting the original URLRequest to communicate over the socks proxy listening on the local TCP port.
    private func startDataTask(
        request: URLRequest,
        localPort: UInt16,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        // Copy the URL request and rewrite the host and port to point to the socks5 forwarding proxy instance
        var newRequest = request

        newRequest.url = request.url.flatMap { url in
            var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
            components?.host = localhost
            components?.port = Int(localPort)
            return components?.url
        }

        let dataTask = urlSession.dataTask(with: newRequest, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
