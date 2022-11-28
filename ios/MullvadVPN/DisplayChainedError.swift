//
//  DisplayChainedError.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import StoreKit

protocol DisplayChainedError {
    var errorChainDescription: String? { get }
}

extension REST.Error: DisplayChainedError {
    var errorChainDescription: String? {
        switch self {
        case let .network(urlError):
            return String(
                format: NSLocalizedString(
                    "NETWORK_ERROR",
                    tableName: "REST",
                    value: "Network error: %@",
                    comment: ""
                ),
                urlError.localizedDescription
            )
        case let .unhandledResponse(statusCode, serverResponse):
            return String(
                format: NSLocalizedString(
                    "SERVER_ERROR",
                    tableName: "REST",
                    value: "Unexpected server response: %1$@ (HTTP status: %2$d)",
                    comment: ""
                ),
                serverResponse?.code.rawValue ?? "(no code)",
                statusCode
            )
        case .createURLRequest:
            return NSLocalizedString(
                "SERVER_REQUEST_ENCODING_ERROR",
                tableName: "REST",
                value: "Failure to create URL request",
                comment: ""
            )
        case .decodeResponse:
            return NSLocalizedString(
                "SERVER_SUCCESS_RESPONSE_DECODING_ERROR",
                tableName: "REST",
                value: "Server response decoding error",
                comment: ""
            )
        case let .transport(error):
            return NSLocalizedString(
                "TRANSPORT_ERROR",
                tableName: "REST",
                value: "Transport error: \(error.localizedDescription)",
                comment: ""
            )
        }
    }
}

extension SKError: LocalizedError {
    public var errorDescription: String? {
        switch code {
        case .unknown:
            return NSLocalizedString(
                "UNKNOWN_ERROR",
                tableName: "StoreKitErrors",
                value: "Unknown error.",
                comment: ""
            )
        case .clientInvalid:
            return NSLocalizedString(
                "CLIENT_INVALID",
                tableName: "StoreKitErrors",
                value: "Client is not allowed to issue the request.",
                comment: ""
            )
        case .paymentCancelled:
            return NSLocalizedString(
                "PAYMENT_CANCELLED",
                tableName: "StoreKitErrors",
                value: "User cancelled the request.",
                comment: ""
            )
        case .paymentInvalid:
            return NSLocalizedString(
                "PAYMENT_INVALID",
                tableName: "StoreKitErrors",
                value: "Invalid purchase identifier.",
                comment: ""
            )
        case .paymentNotAllowed:
            return NSLocalizedString(
                "PAYMENT_NOT_ALLOWED",
                tableName: "StoreKitErrors",
                value: "This device is not allowed to make the payment.",
                comment: ""
            )
        default:
            return localizedDescription
        }
    }
}

extension StorePaymentManagerError: DisplayChainedError {
    var errorChainDescription: String? {
        switch self {
        case .noAccountSet:
            return NSLocalizedString(
                "NO_ACCOUNT_SET_ERROR",
                tableName: "StorePaymentManager",
                value: "Internal error: account is not set.",
                comment: ""
            )

        case let .validateAccount(restError):
            let reason = restError.errorChainDescription ?? ""

            if restError.compareErrorCode(.invalidAccount) {
                return String(
                    format: NSLocalizedString(
                        "INVALID_ACCOUNT_ERROR",
                        tableName: "StorePaymentManager",
                        value: "Cannot add credit to invalid account.",
                        comment: ""
                    ), reason
                )
            } else {
                let reason = restError.errorChainDescription ?? ""

                return String(
                    format: NSLocalizedString(
                        "VALIDATE_ACCOUNT_ERROR",
                        tableName: "StorePaymentManager",
                        value: "Failed to validate account token: %@",
                        comment: ""
                    ), reason
                )
            }

        case let .readReceipt(readReceiptError):
            if readReceiptError is StoreReceiptNotFound {
                return NSLocalizedString(
                    "RECEIPT_NOT_FOUND_ERROR",
                    tableName: "StorePaymentManager",
                    value: "AppStore receipt is not found on disk.",
                    comment: ""
                )
            } else if let storeError = readReceiptError as? SKError {
                return String(
                    format: NSLocalizedString(
                        "REFRESH_RECEIPT_ERROR",
                        tableName: "StorePaymentManager",
                        value: "Cannot refresh the AppStore receipt: %@",
                        comment: ""
                    ),
                    storeError.localizedDescription
                )
            } else {
                return String(
                    format: NSLocalizedString(
                        "READ_RECEIPT_ERROR",
                        tableName: "StorePaymentManager",
                        value: "Cannot read the AppStore receipt from disk: %@",
                        comment: ""
                    ),
                    readReceiptError.localizedDescription
                )
            }

        case let .sendReceipt(restError):
            let reason = restError.errorChainDescription ?? ""
            let errorFormat = NSLocalizedString(
                "SEND_RECEIPT_ERROR",
                tableName: "StorePaymentManager",
                value: "Failed to send the receipt to server: %@",
                comment: ""
            )
            let recoverySuggestion = NSLocalizedString(
                "SEND_RECEIPT_RECOVERY_SUGGESTION",
                tableName: "StorePaymentManager",
                value: "Please retry by using the \"Restore purchases\" button.",
                comment: ""
            )
            var errorString = String(format: errorFormat, reason)
            errorString.append("\n\n")
            errorString.append(recoverySuggestion)
            return errorString

        case let .storePayment(storeError):
            return (storeError as? SKError)?.errorDescription ?? storeError.localizedDescription
        }
    }
}
