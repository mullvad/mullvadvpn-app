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
        try await perform(retryStrategy: retryStrategy, path: "https://ipv6.am.i.mullvad.net/json")
    }

    func getIPV4(retryStrategy: REST.RetryStrategy) async throws -> IPV4ConnectionData {
        try await perform(retryStrategy: retryStrategy, path: "https://ipv4.am.i.mullvad.net/json")
    }

    private func perform<T: Decodable>(retryStrategy: REST.RetryStrategy, path: String) async throws -> T {
        let delayIterator = retryStrategy.makeDelayIterator()
        for _ in 0 ..< retryStrategy.maxRetryCount {
            do {
                return try await perform(path: path)
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
        return try await perform(path: path)
    }

    private func perform<T: Decodable>(path: String) async throws -> T {
        guard let url = URL(string: path) else {
            throw REST.Error.network(URLError(.badURL))
        }
        do {
            let (data, response) = try await urlSession.data(from: url)
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
        let ip: T?
        let exitIP: Bool

        enum CodingKeys: String, CodingKey {
            case ip, exitIP = "mullvad_exit_ip"
        }

        init(ip: T, exitIP: Bool) {
            self.ip = ip
            self.exitIP = exitIP
        }

        init(from decoder: Decoder) throws {
            let values = try decoder.container(keyedBy: CodingKeys.self)
            self.ip = try values.decodeIfPresent(T.self, forKey: .ip)
            self.exitIP = try values.decodeIfPresent(Bool.self, forKey: .exitIP) ?? false
        }

        func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: Self.CodingKeys.self)
            try container.encode(self.ip, forKey: Self.CodingKeys.ip)
            try container.encode(self.exitIP, forKey: Self.CodingKeys.exitIP)
        }

        static func == (
            lhs: OutgoingConnectionProxy.OutgoingConnectionData<T>,
            rhs: OutgoingConnectionProxy.OutgoingConnectionData<T>
        ) -> Bool {
            lhs.ip?.rawValue == rhs.ip?.rawValue
        }
    }
}
