//
//  RESTCoding.swift
//  RESTCoding
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    enum Coding {}
}

extension REST.Coding {
    /// Returns a JSON encoder used by REST API.
    static func makeJSONEncoder() -> JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dataEncodingStrategy = .base64
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }

    /// Returns a JSON decoder used by REST API.
    static func makeJSONDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dataDecodingStrategy = .base64

        let iso8601Formatter = ISO8601DateFormatter()

        // Setup additional formatter to account for fractional seconds returned
        // by some of the API calls.
        lazy var iso8601WithSubSecondsFormatter: ISO8601DateFormatter = {
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions.insert(.withFractionalSeconds)
            return formatter
        }()

        decoder.dateDecodingStrategy = .custom({ decoder in
            let container = try decoder.singleValueContainer()
            let value = try container.decode(String.self)

            let date = iso8601Formatter.date(from: value) ??
                iso8601WithSubSecondsFormatter.date(from: value)

            switch date {
            case .some(let parsedDate):
                return parsedDate

            case .none:
                throw DecodingError.dataCorruptedError(
                    in: container,
                    debugDescription: "Expected date string to be RFC3339 or ISO8601-formatted."
                )
            }
        })

        return decoder
    }
}
