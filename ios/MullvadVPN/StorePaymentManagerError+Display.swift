//
//  StorePaymentManagerError+Display.swift
//  MullvadVPN
//
//  Created by pronebird on 17/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import StoreKit

extension StorePaymentManagerError: DisplayError {
    var displayErrorDescription: String? {
        switch self {
        case .noAccountSet:
            return NSLocalizedString(
                "INTERNAL_ERROR",
                tableName: "StorePaymentManager",
                value: "Internal error.",
                comment: ""
            )

        case let .validateAccount(restError):
            let reason = restError.displayErrorDescription ?? ""

            return String(
                format: NSLocalizedString(
                    "VALIDATE_ACCOUNT_ERROR",
                    tableName: "StorePaymentManager",
                    value: "Failed to validate account number: %@",
                    comment: ""
                ), reason
            )

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
                return NSLocalizedString(
                    "READ_RECEIPT_ERROR",
                    tableName: "StorePaymentManager",
                    value: "Cannot read the AppStore receipt from disk",
                    comment: ""
                )
            }

        case let .sendReceipt(restError):
            let reason = restError.displayErrorDescription ?? ""
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
