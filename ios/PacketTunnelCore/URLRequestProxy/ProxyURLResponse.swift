//
//  ProxyURLResponse.swift
//  PacketTunnelCore
//
//  Created by pronebird on 20/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

/// Struct describing serializable URLResponse data.
public struct ProxyURLResponse: Codable, Sendable {
    public let data: Data?
    public let response: HTTPURLResponseWrapper?
    public let error: URLErrorWrapper?

    public init(data: Data?, response: URLResponse?, error: Error?) {
        self.data = data
        self.response = response.flatMap { HTTPURLResponseWrapper($0) }
        self.error = error.flatMap { URLErrorWrapper($0) }
    }
}

public struct URLErrorWrapper: Codable, Sendable {
    public let code: Int?
    public let localizedDescription: String

    public init?(_ error: Error) {
        localizedDescription = error.localizedDescription
        code = (error as? URLError)?.errorCode
    }

    public var originalError: Error? {
        guard let code else { return nil }

        return URLError(URLError.Code(rawValue: code))
    }
}

public struct HTTPURLResponseWrapper: Codable, Sendable {
    public let url: URL?
    public let statusCode: Int
    public let headerFields: [String: String]?

    public init?(_ response: URLResponse) {
        guard let response = response as? HTTPURLResponse else { return nil }

        url = response.url
        statusCode = response.statusCode
        headerFields = Dictionary(
            uniqueKeysWithValues: response.allHeaderFields.map { ("\($0)", "\($1)") }
        )
    }

    public var originalResponse: HTTPURLResponse? {
        guard let url else { return nil }

        return HTTPURLResponse(
            url: url,
            statusCode: statusCode,
            httpVersion: nil,
            headerFields: headerFields
        )
    }
}
