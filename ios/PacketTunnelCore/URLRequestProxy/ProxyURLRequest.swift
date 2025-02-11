//
//  ProxyURLRequest.swift
//  PacketTunnelCore
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

/// Struct describing serializable URLRequest data.
public struct ProxyURLRequest: Codable {
    public let id: UUID
    public let url: URL
    public let method: String?
    public let httpBody: Data?
    public let httpHeaders: [String: String]?

    public var urlRequest: URLRequest {
        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = method
        urlRequest.httpBody = httpBody
        urlRequest.allHTTPHeaderFields = httpHeaders
        return urlRequest
    }

    public init?(id: UUID, urlRequest: URLRequest) {
        guard let urlRequestUrl = urlRequest.url else { return nil }

        self.id = id
        url = urlRequestUrl
        method = urlRequest.httpMethod
        httpBody = urlRequest.httpBody
        httpHeaders = urlRequest.allHTTPHeaderFields
    }
}

public struct ProxyAPIRequest: Codable {
    public let id: UUID
    public let request: MullvadApiRequest

    public init(id: UUID, request: MullvadApiRequest) {
        self.id = id
        self.request = request
    }
}
