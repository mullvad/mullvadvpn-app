//
//  RESTCreateApplePaymentResponse.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-04.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadREST

extension REST.CreateApplePaymentResponse {
    enum Context {
        case purchase
        case restoration
    }

    func alertTitle(context: Context) -> String {
        switch context {
        case .purchase:
            return NSLocalizedString(
                "TIME_ADDED_ALERT_SUCCESS_TITLE",
                tableName: "REST",
                value: "Thanks for your purchase",
                comment: ""
            )
        case .restoration:
            return NSLocalizedString(
                "RESTORE_PURCHASES_ALERT_TITLE",
                tableName: "REST",
                value: "Restore purchases",
                comment: ""
            )
        }
    }

    func alertMessage(context: Context) -> String {
        switch context {
        case .purchase:
            return String(
                format: NSLocalizedString(
                    "TIME_ADDED_ALERT_SUCCESS_MESSAGE",
                    tableName: "REST",
                    value: "%@ have been added to your account",
                    comment: ""
                ),
                formattedTimeAdded ?? ""
            )
        case .restoration:
            switch self {
            case .noTimeAdded:
                return NSLocalizedString(
                    "RESTORE_PURCHASES_ALERT_NO_TIME_ADDED_MESSAGE",
                    tableName: "REST",
                    value: "Your previous purchases have already been added to this account.",
                    comment: ""
                )
            case .timeAdded:
                return String(
                    format: NSLocalizedString(
                        "RESTORE_PURCHASES_ALERT_TIME_ADDED_MESSAGE",
                        tableName: "REST",
                        value: "%@ have been added to your account",
                        comment: ""
                    ),
                    formattedTimeAdded ?? ""
                )
            }
        }
    }
}
