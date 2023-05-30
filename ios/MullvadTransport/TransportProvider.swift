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
                logger.error("Could not get shadow socks bridge information.")
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
            logger.error(error: error)
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
        let transportSwitchErrors: [URLError.Code] = [
            .cancelled,
            .notConnectedToInternet,
            .internationalRoamingOff,
            .callIsActive,
        ]

        let failureCompletionHandler: (Data?, URLResponse?, Error?)
            -> Void = { [weak self] data, response, maybeError in
                guard let self else { return }
                if let error = maybeError as? URLError,
                   transportSwitchErrors.contains(error.code) == false
                {
                    parallelRequestsMutex.lock()
                    // Guarantee that the transport strategy switches mode only once when parallel requests fail at
                    // the same time.
                    if currentStrategy == transportStrategy {
                        transportStrategy.didFail()
                        currentTransport = nil
                    }
                    parallelRequestsMutex.unlock()
                }
                completion(data, response, maybeError)
            }

        return transport.sendRequest(request, completion: failureCompletionHandler)
    }

    func makeTransport() -> RESTTransport? {
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
