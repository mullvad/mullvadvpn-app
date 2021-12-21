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

    let rawValue: String
    init(rawValue: String) {
        self.rawValue = rawValue.uppercased()
    }
}

enum HTTPStatus {
    static let notModified = 304

    static func isSuccess(_ code: Int) -> Bool {
        return (200..<300).contains(code)
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

extension HTTPURLResponse {
    func value(forCaseInsensitiveHTTPHeaderField headerField: String) -> String? {
        if #available(iOS 13.0, *) {
            return self.value(forHTTPHeaderField: headerField)
        } else {
            for case let key as String in self.allHeaderFields.keys {
                if case .orderedSame = key.caseInsensitiveCompare(headerField) {
                    return self.allHeaderFields[key] as? String
                }
            }
            return nil
        }
    }
}
