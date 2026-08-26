// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

/// HTTP method
struct HTTPMethod: RawRepresentable {
    static let get = HTTPMethod(rawValue: "GET")
    static let post = HTTPMethod(rawValue: "POST")
    static let delete = HTTPMethod(rawValue: "DELETE")
    static let put = HTTPMethod(rawValue: "PUT")
    static let head = HTTPMethod(rawValue: "HEAD")

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
        (200..<300).contains(code)
    }

    let rawValue: Int
    init(rawValue: Int) {
        self.rawValue = rawValue
    }

    var isSuccess: Bool {
        Self.isSuccess(rawValue)
    }
}

/// HTTP headers
enum HTTPHeader {
    static let host = "Host"
    static let authorization = "Authorization"
    static let contentType = "Content-Type"
    static let ifNoneMatch = "If-None-Match"
    static let userAgent = "User-Agent"
}
