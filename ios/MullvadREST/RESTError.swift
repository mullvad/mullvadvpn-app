//
//  RESTError.swift
//  RESTError
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    /// An error type returned by REST API classes.
    public enum Error: LocalizedError, WrappingError {
        /// Failure to create URL request.
        case createURLRequest(Swift.Error)

        /// Network failure.
        case network(URLError)

        /// Failure to handle response.
        case unhandledResponse(_ statusCode: Int, _ serverResponse: ServerErrorResponse?)

        /// Failure to decode server response.
        case decodeResponse(Swift.Error)

        /// Failure to transit URL request via selected transport implementation.
        case transport(Swift.Error)

        public var errorDescription: String? {
            switch self {
            case .createURLRequest:
                return "Failure to create URL request."

            case .network:
                return "Network error."

            case let .unhandledResponse(statusCode, serverResponse):
                var str = "Failure to handle server response: HTTP/\(statusCode)."

                if let code = serverResponse?.code {
                    str += " Error code: \(code.rawValue)."
                }

                if let detail = serverResponse?.detail {
                    str += " Detail: \(detail)"
                }

                return str

            case .decodeResponse:
                return "Failure to decode response."

            case .transport:
                return "Transport error."
            }
        }

        public var underlyingError: Swift.Error? {
            switch self {
            case let .network(error):
                return error
            case let .createURLRequest(error):
                return error
            case let .decodeResponse(error):
                return error
            case let .transport(error):
                return error
            case .unhandledResponse:
                return nil
            }
        }

        public func compareErrorCode(_ code: ServerResponseCode) -> Bool {
            if case let .unhandledResponse(_, serverResponse) = self {
                return serverResponse?.code == code
            } else {
                return false
            }
        }
    }

    public struct ServerErrorResponse: Decodable {
        public let code: ServerResponseCode
        public let detail: String?

        private enum CodingKeys: String, CodingKey {
            case code, detail, error
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            let rawValue = try container.decode(String.self, forKey: .code)

            code = ServerResponseCode(rawValue: rawValue)
            detail = try container.decodeIfPresent(String.self, forKey: .detail)
                ?? container.decodeIfPresent(String.self, forKey: .error)
        }
    }

    public struct ServerResponseCode: RawRepresentable, Equatable {
        public static let invalidAccount = ServerResponseCode(rawValue: "INVALID_ACCOUNT")
        public static let keyLimitReached = ServerResponseCode(rawValue: "KEY_LIMIT_REACHED")
        public static let publicKeyNotFound = ServerResponseCode(rawValue: "PUBKEY_NOT_FOUND")
        public static let publicKeyInUse = ServerResponseCode(rawValue: "PUBKEY_IN_USE")
        public static let maxDevicesReached = ServerResponseCode(rawValue: "MAX_DEVICES_REACHED")
        public static let invalidAccessToken = ServerResponseCode(rawValue: "INVALID_ACCESS_TOKEN")
        public static let deviceNotFound = ServerResponseCode(rawValue: "DEVICE_NOT_FOUND")

        public let rawValue: String
        public init(rawValue: String) {
            self.rawValue = rawValue
        }
    }

    public struct NoTransportError: LocalizedError {
        public var errorDescription: String? {
            return "Transport is not configured."
        }
    }
}
