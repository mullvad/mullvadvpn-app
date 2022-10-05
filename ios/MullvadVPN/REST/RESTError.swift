//
//  RESTError.swift
//  RESTError
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    /// An error type returned by REST API classes.
    enum Error: LocalizedError, WrappingError {
        /// A failure to create URL request.
        case createURLRequest(Swift.Error)

        /// A failure during networking.
        case network(URLError)

        /// A failure to handle response.
        case unhandledResponse(_ statusCode: Int, _ serverResponse: ServerErrorResponse?)

        /// A failure to decode server response.
        case decodeResponse(Swift.Error)

        /// A failure to get the url error from server response, treating error like any Error.
        case transport(Swift.Error)

        var errorDescription: String? {
            switch self {
            case let .createURLRequest(error):
                return "Failure to create URL request: \(error.localizedDescription)."
            case let .network(error):
                return "Network error: \(error.localizedDescription)."
            case let .unhandledResponse(statusCode, serverResponse):
                var str = "Failure to handle server response: HTTP/\(statusCode)."

                if let code = serverResponse?.code {
                    str += " Error code: \(code.rawValue)."
                }

                if let detail = serverResponse?.detail {
                    str += " Detail: \(detail)"
                }

                return str
            case let .decodeResponse(error):
                return "Failure to decode URL response data: \(error.localizedDescription)."
            case .transport:
                return "Failure to handle, unknown error happened."
            }
        }

        var underlyingError: Swift.Error? {
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

        func compareErrorCode(_ code: ServerResponseCode) -> Bool {
            if case let .unhandledResponse(_, serverResponse) = self {
                return serverResponse?.code == code
            } else {
                return false
            }
        }
    }

    struct ServerErrorResponse: Decodable {
        let code: ServerResponseCode
        let detail: String?

        private enum CodingKeys: String, CodingKey {
            case code, detail, error
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            let rawValue = try container.decode(String.self, forKey: .code)

            code = ServerResponseCode(rawValue: rawValue)
            detail = try container.decodeIfPresent(String.self, forKey: .detail)
                ?? container.decodeIfPresent(String.self, forKey: .error)
        }
    }

    struct ServerResponseCode: RawRepresentable, Equatable {
        static let invalidAccount = ServerResponseCode(rawValue: "INVALID_ACCOUNT")
        static let keyLimitReached = ServerResponseCode(rawValue: "KEY_LIMIT_REACHED")
        static let publicKeyNotFound = ServerResponseCode(rawValue: "PUBKEY_NOT_FOUND")
        static let publicKeyInUse = ServerResponseCode(rawValue: "PUBKEY_IN_USE")
        static let maxDevicesReached = ServerResponseCode(rawValue: "MAX_DEVICES_REACHED")
        static let invalidAccessToken = ServerResponseCode(rawValue: "INVALID_ACCESS_TOKEN")
        static let deviceNotFound = ServerResponseCode(rawValue: "DEVICE_NOT_FOUND")

        let rawValue: String
        init(rawValue: String) {
            self.rawValue = rawValue
        }
    }
}
