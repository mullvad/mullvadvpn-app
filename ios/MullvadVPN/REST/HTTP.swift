//
//  HTTP.swift
//  HTTP
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// HTTP method
struct HTTPMethod: RawRepresentable {
    static let get = HTTPMethod(rawValue: "GET")
    static let post = HTTPMethod(rawValue: "POST")
    static let delete = HTTPMethod(rawValue: "DELETE")
    static let put = HTTPMethod(rawValue: "PUT")

    let rawValue: String
    init(rawValue: String) {
        self.rawValue = rawValue.uppercased()
    }
}

struct HTTPStatus: RawRepresentable, Equatable {
    static let notModified = HTTPStatus(rawValue: 304)
    static let badRequest = HTTPStatus(rawValue: 400)
    static let notFound = HTTPStatus(rawValue: 404)

    static func isSuccess(_ code: Int) -> Bool {
        return (200 ..< 300).contains(code)
    }

    let rawValue: Int
    init(rawValue: Int) {
        self.rawValue = rawValue
    }

    var isSuccess: Bool {
        return Self.isSuccess(rawValue)
    }
}

/// HTTP headers
enum HTTPHeader {
    static let host = "Host"
    static let authorization = "Authorization"
    static let contentType = "Content-Type"
    static let etag = "ETag"
    static let ifNoneMatch = "If-None-Match"
}
