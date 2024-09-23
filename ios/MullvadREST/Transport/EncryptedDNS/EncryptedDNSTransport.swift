//
//  EncryptedDNSTransport.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-09-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadRustRuntime
import MullvadTypes

public final class EncryptedDNSTransport: RESTTransport {
    public var name: String {
        "encrypted-dns-url-session"
    }

    /// The `URLSession` used to send requests via `encryptedDNSProxy`
    public let urlSession: URLSession

    public init(
        urlSession: URLSession,
        addressCache: REST.AddressCache
    ) {
        self.urlSession = urlSession
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, (any Error)?) -> Void
    ) -> any Cancellable {
        // TODO: Start proxy once the backend is integrated into the Swift code.
        let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
