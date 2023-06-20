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
    private let shadowsocksCache: ShadowsocksConfigurationCache
    private var transportStrategy: TransportStrategy

    private var currentTransport: RESTTransport?
    private let parallelRequestsMutex = NSLock()

    public init(
        urlSessionTransport: URLSessionTransport,
        relayCache: RelayCache,
        addressCache: REST.AddressCache,
        shadowsocksCache: ShadowsocksConfigurationCache,
        transportStrategy: TransportStrategy = .init()
    ) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
        self.addressCache = addressCache
        self.shadowsocksCache = shadowsocksCache
        self.transportStrategy = transportStrategy
    }

    // MARK: -

    // MARK: RESTTransport implementation

    private func shadowsocksTransport() -> RESTTransport? {
        do {
            let shadowsocksConfiguration = try shadowsocksConfiguration()

            let shadowsocksURLSession = urlSessionTransport.urlSession
            let shadowsocksTransport = URLSessionShadowsocksTransport(
                urlSession: shadowsocksURLSession,
                shadowsocksConfiguration: shadowsocksConfiguration,
                addressCache: addressCache
            )

            return shadowsocksTransport
        } catch {
            logger.error(error: error, message: "Failed to produce shadowsocks configuration.")
            return nil
        }
    }

    /// Returns the last used shadowsocks configuration, otherwise a new randomized configuration.
    private func shadowsocksConfiguration() throws -> ShadowsocksConfiguration {
        // If a previous shadowsocks configuration was in cache, return it directly.
        do {
            return try shadowsocksCache.read()
        } catch {
            // There is no previous configuration either if this is the first time this code ran
            // Or because the previous shadowsocks configuration was invalid, therefore generate a new one.
            return try makeNewShadowsocksConfiguration()
        }
    }

    /// Returns a randomly selected shadowsocks configuration.
    private func makeNewShadowsocksConfiguration() throws -> ShadowsocksConfiguration {
        let cachedRelays = try relayCache.read()
        let bridgeAddress = RelaySelector.getShadowsocksRelay(relays: cachedRelays.relays)?.ipv4AddrIn
        let bridgeConfiguration = RelaySelector.getShadowsocksTCPBridge(relays: cachedRelays.relays)

        guard let bridgeAddress, let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        let newConfiguration = ShadowsocksConfiguration(
            bridgeAddress: bridgeAddress,
            bridgePort: bridgeConfiguration.port,
            password: bridgeConfiguration.password,
            cipher: bridgeConfiguration.cipher
        )

        do {
            try shadowsocksCache.write(newConfiguration)
        } catch {
            logger.error(error: error, message: "Failed to persist shadowsocks cache.")
        }

        return newConfiguration
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

        let failureCompletionHandler: (Data?, URLResponse?, Error?)
            -> Void = { [weak self] data, response, maybeError in
                guard let self else { return }

                if let error = maybeError as? URLError, error.shouldResetNetworkTransport {
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
    private func makeTransport() -> RESTTransport? {
        if currentTransport == nil {
            switch transportStrategy.connectionTransport() {
            case .useShadowsocks:
                currentTransport = shadowsocksTransport()
            case .useURLSession:
                currentTransport = urlSessionTransport
            }
        }
        return currentTransport
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
