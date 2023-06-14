//
//  TransportProvider.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-05-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadREST
import MullvadTypes
import RelayCache
import RelaySelector

public final class TransportProvider: RESTTransport {
    private let urlSessionTransport: URLSessionTransport
    private let relayCache: RelayCache
    private let logger = Logger(label: "TransportProvider")
    private let addressCache: REST.AddressCache
    private var transportStrategy: TransportStrategy

    private var currentTransport: RESTTransport?
    private let parallelRequestsMutex = NSLock()

    public init(
        urlSessionTransport: URLSessionTransport,
        relayCache: RelayCache,
        addressCache: REST.AddressCache,
        transportStrategy: TransportStrategy = .init()
    ) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
        self.addressCache = addressCache
        self.transportStrategy = transportStrategy
    }

    // MARK: -

    // MARK: RESTTransport implementation

    private func transport() -> RESTTransport {
        urlSessionTransport
    }

    private func shadowsocksTransport() -> RESTTransport? {
        do {
            let cachedRelays = try relayCache.read()
            let shadowsocksConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)
            let shadowsocksBridgeRelay = RelaySelector.getShadowsocksRelay(relays: cachedRelays.relays)

            guard let shadowsocksConfiguration,
                  let shadowsocksBridgeRelay
            else {
                logger.error("Failed to get shadow socks bridge information.")
                return nil
            }

            let shadowsocksURLSession = urlSessionTransport.urlSession
            let shadowsocksTransport = URLSessionShadowsocksTransport(
                urlSession: shadowsocksURLSession,
                shadowsocksConfiguration: shadowsocksConfiguration,
                shadowsocksBridgeRelay: shadowsocksBridgeRelay,
                addressCache: addressCache
            )

            return shadowsocksTransport
        } catch {
            logger.error(error: error, message: "Failed to create shadowsocks transport.")
        }
        return nil
    }

    // MARK: -

    // MARK: RESTTransport implementation

    public var name: String { currentTransport?.name ?? "TransportProvider" }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        parallelRequestsMutex.lock()
        defer {
            parallelRequestsMutex.unlock()
        }

        let currentStrategy = transportStrategy
        guard let transport = makeTransport() else { return AnyCancellable() }
        let nonSwitchErrorCodes: [URLError.Code] = [
            .cancelled,
            .notConnectedToInternet,
            .internationalRoamingOff,
            .callIsActive,
        ]

        let failureCompletionHandler: (Data?, URLResponse?, Error?)
            -> Void = { [weak self] data, response, maybeError in
                guard let self else { return }

                if let error = maybeError as? URLError,
                   nonSwitchErrorCodes.contains(error.code) == false
                {
                    resetTransportMatching(currentStrategy)
                }
                completion(data, response, maybeError)
            }

        return transport.sendRequest(request, completion: failureCompletionHandler)
    }

    /// When several requests fail at the same time,  prevents the `transportStrategy` from switching multiple times.
    ///
    /// The `strategy` is checked against the `transportStrategy`. When several requests are made and fail in parallel,
    /// only the first failure will pass the equality check.
    /// Subsequent failures will not cause the strategy to change several times in a quick fashion.
    /// - Parameter strategy: The strategy object used when sending a request
    private func resetTransportMatching(_ strategy: TransportStrategy) {
        parallelRequestsMutex.lock()
        if strategy == transportStrategy {
            transportStrategy.didFail()
            currentTransport = nil
        }
        parallelRequestsMutex.unlock()
    }

    private func makeTransport() -> RESTTransport? {
        if currentTransport == nil {
            switch transportStrategy.connectionTransport() {
            case .useShadowsocks:
                currentTransport = shadowsocksTransport()
            case .useURLSession:
                currentTransport = transport()
            }
        }
        return currentTransport
    }
}
