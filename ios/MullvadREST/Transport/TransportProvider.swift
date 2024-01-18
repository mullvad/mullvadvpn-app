//
//  TransportProvider.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-05-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadTypes

public final class TransportProvider: RESTTransportProvider {
    private let urlSessionTransport: URLSessionTransport
    private let addressCache: REST.AddressCache
    private var transportStrategy: TransportStrategy
    private var currentTransport: RESTTransport?
    private var currentTransportType: TransportStrategy.Transport
    private let parallelRequestsMutex = NSLock()

    public init(
        urlSessionTransport: URLSessionTransport,
        addressCache: REST.AddressCache,
        transportStrategy: TransportStrategy
    ) {
        self.urlSessionTransport = urlSessionTransport
        self.addressCache = addressCache
        self.transportStrategy = transportStrategy
        self.currentTransportType = transportStrategy.connectionTransport()
    }

    public func makeTransport() -> RESTTransport? {
        parallelRequestsMutex.withLock {
            guard let actualTransport = makeTransportInner() else { return nil }

            let currentStrategy = transportStrategy
            return TransportWrapper(wrapped: actualTransport) { [weak self] error in
                if let error = error as? URLError, error.shouldResetNetworkTransport {
                    self?.resetTransportMatching(currentStrategy)
                }
            }
        }
    }

    /// When several requests fail at the same time,  prevents the `transportStrategy` from switching multiple times.
    ///
    /// The `strategy` is checked against the `transportStrategy`. When several requests are made and fail in parallel,
    /// only the first failure will pass the equality check.
    /// Subsequent failures will not cause the strategy to change several times in a quick fashion.
    /// - Parameter strategy: The strategy object used when sending a request
    private func resetTransportMatching(_ strategy: TransportStrategy) {
        parallelRequestsMutex.lock()
        defer { parallelRequestsMutex.unlock() }

        if strategy == transportStrategy {
            transportStrategy.didFail()
            currentTransport = nil
        }
    }

    /// Sets and returns the `currentTransport` according to the suggestion from `transportStrategy`
    ///
    /// > Warning: Do not  lock the `parallelRequestsMutex` in this method
    ///
    /// - Returns: A `RESTTransport` object to make a connection
    private func makeTransportInner() -> RESTTransport? {
        if currentTransport == nil || shouldNotReuseCurrentTransport {
            currentTransportType = transportStrategy.connectionTransport()
            switch currentTransportType {
            case .direct:
                currentTransport = urlSessionTransport
            case let .shadowsocks(configuration):
                currentTransport = ShadowsocksTransport(
                    urlSession: urlSessionTransport.urlSession,
                    configuration: configuration,
                    addressCache: addressCache
                )
            case let .socks5(configuration):
                currentTransport = URLSessionSocks5Transport(
                    urlSession: urlSessionTransport.urlSession,
                    configuration: configuration,
                    addressCache: addressCache
                )
            case .none:
                currentTransport = nil
            }
        }
        return currentTransport
    }

    /// The `Main` allows modifications to access methods through the UI.
    /// The `TransportProvider` relies on a `CurrentTransport` value set during build time or network error.
    /// To ensure  both process `Packet Tunnel` and `Main`  uses the latest changes, the `TransportProvider` compares the `transportType` with the latest value in the cache and reuse it if it's still valid .
    private var shouldNotReuseCurrentTransport: Bool {
        currentTransportType != transportStrategy.connectionTransport()
    }
}

private extension URLError {
    /// Whether the transport selection should be reset.
    ///
    /// `true` if the network request
    ///  * Was not cancelled
    ///  * Was not done during a phone call
    ///  * Was made when internet connection was available
    ///  * Was made in a context with data roaming, but international roaming was turned off
    var shouldResetNetworkTransport: Bool {
        code != .cancelled &&
            code != .notConnectedToInternet &&
            code != .internationalRoamingOff &&
            code != .callIsActive
    }
}

/// Interstitial implementation of `RESTTransport` that intercepts the completion of the wrapped transport.
private struct TransportWrapper: RESTTransport {
    let wrapped: RESTTransport
    let onComplete: (Error?) -> Void

    var name: String {
        return wrapped.name
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        return wrapped.sendRequest(request) { data, response, error in
            onComplete(error)
            completion(data, response, error)
        }
    }
}
