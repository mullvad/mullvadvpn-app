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
    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> OutgoingConnectionProxy.IPV6ConnectionData
    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> OutgoingConnectionProxy.IPV4ConnectionData
}

final class OutgoingConnectionProxy: OutgoingConnectionHandling {
    let urlSession: URLSession

    init(urlSession: URLSession) {
        self.urlSession = urlSession
    }

    func getIPV6(retryStrategy: REST.RetryStrategy) async throws -> IPV6ConnectionData {
        try await perform(retryStrategy: retryStrategy, host: REST.ipV6APIHostname)
    }

    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData {
        try await perform(retryStrategy: retryStrategy, host: REST.ipV4APIHostname)
    }

    private func perform<T: Decodable>(retryStrategy: REST.RetryStrategy, host: String) async throws -> T {
        let delayIterator = retryStrategy.makeDelayIterator()
        for _ in 0 ..< retryStrategy.maxRetryCount {
            do {
                return try await perform(host: host)
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
        return try await perform(host: host)
    }

    private func perform<T: Decodable>(host: String) async throws -> T {
        var urlComponents = URLComponents()
        urlComponents.scheme = "https"
        urlComponents.host = host
        urlComponents.path = "/json"

        guard let url = urlComponents.url else {
            throw REST.Error.network(URLError(.badURL))
        }
        do {
            let request = URLRequest(
                url: url,
                cachePolicy: .useProtocolCachePolicy,
                timeoutInterval: REST.defaultAPINetworkTimeout.timeInterval
            )
            let (data, response) = try await urlSession.data(for: request)
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

        } catch {
            throw error
        }
    }
}

extension OutgoingConnectionProxy {
    typealias IPV4ConnectionData = OutgoingConnectionData<IPv4Address>
    typealias IPV6ConnectionData = OutgoingConnectionData<IPv6Address>
    typealias IPAddressType = Codable & IPAddress

    // MARK: - OutgoingConnectionData

    struct OutgoingConnectionData<T: IPAddressType>: Codable, Equatable {
        let ip: T
        let exitIP: Bool

        enum CodingKeys: String, CodingKey {
            case ip, exitIP = "mullvad_exit_ip"
        }

        static func == (lhs: Self, rhs: Self) -> Bool {
            lhs.ip.rawValue == rhs.ip.rawValue && lhs.exitIP == rhs.exitIP
        }
    }
}
