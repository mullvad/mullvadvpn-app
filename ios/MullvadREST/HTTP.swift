//
//  HTTP.swift
//  HTTP
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// HTTP method
public struct HTTPMethod: RawRepresentable {
    public static let get = HTTPMethod(rawValue: "GET")
    public static let post = HTTPMethod(rawValue: "POST")
    public static let delete = HTTPMethod(rawValue: "DELETE")
    public static let put = HTTPMethod(rawValue: "PUT")

    public let rawValue: String
    public init(rawValue: String) {
        self.rawValue = rawValue.uppercased()
    }
}

public struct HTTPStatus: RawRepresentable, Equatable {
    public static let notModified = HTTPStatus(rawValue: 304)
    public static let badRequest = HTTPStatus(rawValue: 400)
    public static let notFound = HTTPStatus(rawValue: 404)

    public static func isSuccess(_ code: Int) -> Bool {
        return (200 ..< 300).contains(code)
    }

    public let rawValue: Int
    public init(rawValue: Int) {
        self.rawValue = rawValue
    }

    public var isSuccess: Bool {
        return Self.isSuccess(rawValue)
    }
}

/// HTTP headers
public enum HTTPHeader {
    static let host = "Host"
    static let authorization = "Authorization"
    static let contentType = "Content-Type"
    static let etag = "ETag"
    static let ifNoneMatch = "If-None-Match"
}
