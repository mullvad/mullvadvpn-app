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

    var urlRequest: URLRequest {
        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = method
        urlRequest.httpBody = httpBody
        urlRequest.allHTTPHeaderFields = httpHeaders
        return urlRequest
    }

    init(id: UUID, urlRequest: URLRequest) throws {
        guard let url = urlRequest.url else {
            throw InvalidURLRequestError()
        }

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

    init(data: Data?, response: URLResponse?, error: Error?) {
        self.data = data
        self.response = response.flatMap { HTTPURLResponseWrapper($0) }
        self.error = error.flatMap { URLErrorWrapper($0) }
    }
}

struct URLErrorWrapper: Codable {
    let code: Int?
    let localizedDescription: String

    init?(_ error: Error) {
        localizedDescription = error.localizedDescription
        code = (error as? URLError)?.errorCode
    }

    var originalError: Error? {
        guard let code = code else { return nil }

        return URLError(URLError.Code(rawValue: code))
    }
}

struct HTTPURLResponseWrapper: Codable {
    let url: URL?
    let statusCode: Int
    let headerFields: [String: String]?

    init?(_ response: URLResponse) {
        guard let response = response as? HTTPURLResponse else { return nil }

        url = response.url
        statusCode = response.statusCode
        headerFields = Dictionary(
            uniqueKeysWithValues: response.allHeaderFields.map { ("\($0)", "\($1)") }
        )
    }

    var originalResponse: HTTPURLResponse? {
        guard let url = url else { return nil }

        return HTTPURLResponse(
            url: url,
            statusCode: statusCode,
            httpVersion: nil,
            headerFields: headerFields
        )
    }
}

struct InvalidURLRequestError: LocalizedError {
    var errorDescription: String? {
        return "Invalid URLRequest URL."
    }
}
