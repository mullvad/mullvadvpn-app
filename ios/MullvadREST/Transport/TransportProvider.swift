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
    private let relayCache: RelayCacheProtocol
    private let logger = Logger(label: "TransportProvider")
    private let addressCache: REST.AddressCache
    private let shadowsocksCache: ShadowsocksConfigurationCache
    private var transportStrategy: TransportStrategy

    private var currentTransport: RESTTransport?
    private let parallelRequestsMutex = NSLock()
    private var relayConstraints = RelayConstraints()
    private let constraintsUpdater: RelayConstraintsUpdater

    public init(
        urlSessionTransport: URLSessionTransport,
        relayCache: RelayCacheProtocol,
        addressCache: REST.AddressCache,
        shadowsocksCache: ShadowsocksConfigurationCache,
        transportStrategy: TransportStrategy,
        constraintsUpdater: RelayConstraintsUpdater
    ) {
        self.urlSessionTransport = urlSessionTransport
        self.relayCache = relayCache
        self.addressCache = addressCache
        self.shadowsocksCache = shadowsocksCache
        self.transportStrategy = transportStrategy
        self.constraintsUpdater = constraintsUpdater
        constraintsUpdater.onNewConstraints = { [weak self] newConstraints in
            self?.parallelRequestsMutex.lock()
            defer {
                self?.parallelRequestsMutex.unlock()
            }
            self?.relayConstraints = newConstraints
        }
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

    // MARK: -

    private func shadowsocks() -> RESTTransport? {
        do {
            let shadowsocksConfiguration = try shadowsocksConfiguration()

            let shadowsocksURLSession = urlSessionTransport.urlSession
            let shadowsocksTransport = ShadowsocksTransport(
                urlSession: shadowsocksURLSession,
                configuration: shadowsocksConfiguration,
                addressCache: addressCache
            )
            return shadowsocksTransport
        } catch {
            logger.error(error: error, message: "Failed to produce shadowsocks configuration.")
            return nil
        }
    }

    private func socks5() -> RESTTransport? {
        return URLSessionSocks5Transport(
            urlSession: urlSessionTransport.urlSession,
            configuration: Socks5Configuration(address: .ipv4(.loopback), port: 8889),
            addressCache: addressCache
        )
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
        let bridgeConfiguration = RelaySelector.shadowsocksTCPBridge(from: cachedRelays.relays)
        let closestRelay = RelaySelector.closestShadowsocksRelayConstrained(
            by: relayConstraints,
            in: cachedRelays.relays
        )

        guard let bridgeAddress = closestRelay?.ipv4AddrIn, let bridgeConfiguration else { throw POSIXError(.ENOENT) }

        let newConfiguration = ShadowsocksConfiguration(
            address: .ipv4(bridgeAddress),
            port: bridgeConfiguration.port,
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
        if currentTransport == nil {
            switch transportStrategy.connectionTransport() {
            case .useShadowsocks:
                currentTransport = shadowsocks()
            case .useURLSession:
                currentTransport = urlSessionTransport
            case .useSocks5:
                currentTransport = socks5()
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
