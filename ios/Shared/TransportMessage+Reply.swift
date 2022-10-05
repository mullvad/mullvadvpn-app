//
//  RequestModel.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct describing supported transport messages handled by packet tunnel provider.
/// Its a wrapper to send urlRequest to tunnel via Data,
/// and recreate the original request inside the tunnel.
struct TransportMessage: Codable {
    let id: UUID
    let url: URL
    let method: String?
    let httpBody: Data?
    let httpHeaders: [String: String]?

    func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}

extension TransportMessage {
    init(id: UUID, urlRequest: URLRequest) throws {
        guard let url = urlRequest.url else { throw URLError(.badURL) }

        self.id = id
        self.url = url
        self.method = urlRequest.httpMethod
        self.httpBody = urlRequest.httpBody
        self.httpHeaders = urlRequest.allHTTPHeaderFields
    }
}

/// Container type for tunnel transport replies.
/// Its a wrapper for tunnel to respond back to app via Data.
/// It will be decoded it inside the app to get the response from transported end point.
struct TransportMessageReply: Codable {
    let data: Data?
    let response: HTTPURLResponseWrapper?
    let error: URLErrorWrapper?

    func decode(from data: Data) throws -> Self {
        try JSONDecoder().decode(Self.self, from: data)
    }

    func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }

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
