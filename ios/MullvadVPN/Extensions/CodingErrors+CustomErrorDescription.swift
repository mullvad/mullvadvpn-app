//
//  CodingErrors+CustomErrorDescription.swift
//  MullvadVPN
//
//  Created by pronebird on 17/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension DecodingError: CustomErrorDescriptionProtocol {
    public var customErrorDescription: String? {
        switch self {
        case let .typeMismatch(type, context):
            return "Type mismatch, expected \(type) for key at \"\(context.codingPath.codingPathString)\"."

        case let .valueNotFound(_, context):
            return "Value not found at \"\(context.codingPath.codingPathString)\"."

        case let .keyNotFound(codingKey, context):
            return "Key \"\(codingKey.stringValue)\" not found at \"\(context.codingPath.codingPathString)\"."

        case .dataCorrupted:
            return "Data corrupted."

        @unknown default:
            return nil
        }
    }
}

extension EncodingError: CustomErrorDescriptionProtocol {
    public var customErrorDescription: String? {
        switch self {
        case let .invalidValue(_, context):
            return "Invalid value at \"\(context.codingPath.codingPathString)\""

        @unknown default:
            return nil
        }
    }
}

private extension Array where Element == CodingKey {
    var codingPathString: String {
        if isEmpty {
            return "<root>"
        } else {
            return map { $0.stringValue }
                .joined(separator: ".")
        }
    }
}
