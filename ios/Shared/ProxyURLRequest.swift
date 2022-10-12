//
//  ProxyURLRequest.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct describing serializable URLRequest data.
struct ProxyURLRequest: Codable {
    let id: UUID
    let url: URL
    let method: String?
    let httpBody: Data?
    let httpHeaders: [String: String]?

    init(id: UUID, urlRequest: URLRequest) throws {
        guard let url = urlRequest.url else { throw URLError(.badURL) }

        self.id = id
        self.url = url
        method = urlRequest.httpMethod
        httpBody = urlRequest.httpBody
        httpHeaders = urlRequest.allHTTPHeaderFields
    }
}

/// Struct describing serializable URLResponse data.
struct ProxyURLResponse: Codable {
    let data: Data?
    let response: HTTPURLResponseWrapper?
    let error: URLErrorWrapper?

    struct URLErrorWrapper: Codable {
        let code: Int
        let debugDescription: String

        init?(_ error: Error?) throws {
            debugDescription = error.debugDescription

            guard let error = error else { return nil }
            if let error = error as? URLError { code = error.errorCode }
            else { code = -1 }
        }

        func originalError() -> Error? {
            URLError(URLError.Code(rawValue: code))
        }
    }

    struct HTTPURLResponseWrapper: Codable {
        let url: URL?
        let statusCode: Int
        let headerFields: [String: String]?

        init?(_ response: URLResponse?) throws {
            guard let response = response as? HTTPURLResponse else { return nil }

            url = response.url
            statusCode = response.statusCode

            headerFields = Dictionary(
                uniqueKeysWithValues: response.allHeaderFields.map { ("\($0)", "\($1)") }
            )
        }

        func originalResponse() -> HTTPURLResponse? {
            guard let url = url else { return nil }

            return HTTPURLResponse(
                url: url,
                statusCode: statusCode,
                httpVersion: nil,
                headerFields: headerFields
            )
        }
    }
}
