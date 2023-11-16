//
//  OutgoingConnectionProxy.swift
//  MullvadREST
//
//  Created by Mojgan on 2023-10-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Network

protocol OutgoingConnectionHandling {
    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData
    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData
}

final class OutgoingConnectionProxy: OutgoingConnectionHandling {
    private enum ExitIPVersion: String {
        case v4 = "ipv4", v6 = "ipv6"

        var host: String {
            "\(rawValue).am.i.mullvad.net"
        }
    }

    let urlSession: URLSessionProtocol

    init(urlSession: URLSessionProtocol) {
        self.urlSession = urlSession
    }

    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData {
        try await perform(retryStrategy: retryStrategy, version: .v6)
    }

    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData {
        try await perform(retryStrategy: retryStrategy, version: .v4)
    }

    private func perform<T: Decodable>(retryStrategy: REST.RetryStrategy, version: ExitIPVersion) async throws -> T {
        let delayIterator = retryStrategy.makeDelayIterator()
        for _ in 0 ..< retryStrategy.maxRetryCount {
            do {
                return try await perform(host: version.host)
            } catch {
                // ignore if request is cancelled
                if case URLError.cancelled = error {
                    throw error
                } else {
                    // retry with the delay
                    guard let delay = delayIterator.next() else { throw error }
                    let mills = UInt64(max(0, delay.milliseconds))
                    let nanos = mills.saturatingMultiplication(1_000_000)
                    try await Task.sleep(nanoseconds: nanos)
                }
            }
        }
        return try await perform(host: version.host)
    }

    private func perform<T: Decodable>(host: String) async throws -> T {
        var urlComponents = URLComponents()
        urlComponents.scheme = "https"
        urlComponents.host = host
        urlComponents.path = "/json"

        guard let url = urlComponents.url else {
            throw REST.Error.network(URLError(.badURL))
        }
        let request = URLRequest(
            url: url,
            cachePolicy: .useProtocolCachePolicy,
            timeoutInterval: REST.defaultAPINetworkTimeout.timeInterval
        )
        let (data, response) = try await data(from: url)
        guard let httpResponse = response as? HTTPURLResponse else {
            throw REST.Error.network(URLError(.badServerResponse))
        }
        let decoder = JSONDecoder()
        guard (200 ..< 300).contains(httpResponse.statusCode) else {
            throw REST.Error.unhandledResponse(
                httpResponse.statusCode,
                try? decoder.decode(
                    REST.ServerErrorResponse.self,
                    from: data
                )
            )
        }
        let connectionData = try decoder.decode(T.self, from: data)
        return connectionData
    }
}

extension OutgoingConnectionProxy: URLSessionProtocol {
    func data(from url: URL) async throws -> (Data, URLResponse) {
        return try await urlSession.data(from: url)
    }
}
