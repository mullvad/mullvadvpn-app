//
//  CodingErrors+ChainedError.swift
//  MullvadVPN
//
//  Created by pronebird on 17/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension DecodingError: CustomChainedErrorDescriptionProtocol {
    var customErrorDescription: String? {
        switch self {
        case .typeMismatch(let type, let context):
            return "Type mismatch, expected \(type) for key at \"\(context.codingPath.codingPathString)\"."

        case .valueNotFound(_, let context):
            return "Value not found at \"\(context.codingPath.codingPathString)\"."

        case .keyNotFound(let codingKey, let context):
            return "Key \"\(codingKey.stringValue)\" not found at \"\(context.codingPath.codingPathString)\"."

        case .dataCorrupted:
            return "Data corrupted."

        @unknown default:
            return nil
        }
    }
}

extension EncodingError: CustomChainedErrorDescriptionProtocol {
    var customErrorDescription: String? {
        switch self {
        case .invalidValue(_, let context):
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
