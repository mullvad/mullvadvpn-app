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
import MullvadTypes

extension DecodingError: MullvadTypes.CustomErrorDescriptionProtocol {
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

extension EncodingError: MullvadTypes.CustomErrorDescriptionProtocol {
    public var customErrorDescription: String? {
        switch self {
        case let .invalidValue(_, context):
            return "Invalid value at \"\(context.codingPath.codingPathString)\""

        @unknown default:
            return nil
        }
    }
}

private extension [CodingKey] {
    var codingPathString: String {
        if isEmpty {
            return "<root>"
        } else {
            return map { $0.stringValue }
                .joined(separator: ".")
        }
    }
}
