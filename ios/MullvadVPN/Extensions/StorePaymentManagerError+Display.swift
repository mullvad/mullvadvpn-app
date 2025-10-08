//
//  StorePaymentManagerError+Display.swift
//  MullvadVPN
//
//  Created by pronebird on 17/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import StoreKit

extension LegacyStorePaymentManagerError: DisplayError {
    var displayErrorDescription: String? {
        switch self {
        case .noAccountSet:
            return NSLocalizedString("Internal error.", comment: "")

        case let .validateAccount(error):
            let reason = (error as? DisplayError)?.displayErrorDescription ?? ""

            return String(
                format: NSLocalizedString("Invalid account number: %@", comment: ""), reason
            )

        case let .readReceipt(readReceiptError):
            if readReceiptError is StoreReceiptNotFound {
                return NSLocalizedString("App Store receipt is not found on disk.", comment: "")
            } else if let storeError = readReceiptError as? SKError {
                return String(
                    format: NSLocalizedString("Cannot refresh the App Store receipt: %@", comment: ""),
                    storeError.localizedDescription
                )
            } else {
                return NSLocalizedString("Cannot read the App Store receipt from disk", comment: "")
            }

        case let .sendReceipt(error):
            let reason = (error as? DisplayError)?.displayErrorDescription ?? ""
            let errorFormat = NSLocalizedString("Failed to send the receipt to server: %@", comment: "")
            let recoverySuggestion = NSLocalizedString(
                "Please retry by using the \"Restore purchases\" button.",
                comment: ""
            )
            var errorString = String(format: errorFormat, reason)
            errorString.append("\n")
            errorString.append(recoverySuggestion)
            return errorString

        case let .storePayment(storeError):
            guard let error = storeError as? SKError else { return storeError.localizedDescription }
            if error.code.rawValue == 0, error.underlyingErrorChain.map({ $0 as NSError }).first?.code == 825 {
                return SKError(.paymentCancelled).errorDescription
            }
            return SKError(error.code).errorDescription
        }
    }
}
