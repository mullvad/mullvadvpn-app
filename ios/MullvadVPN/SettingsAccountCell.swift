//
//  SettingsAccountCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsAccountCell: SettingsCell {

    var accountExpiryDate: Date? {
        didSet {
            didUpdateAccountExpiry()
        }
    }

    private func didUpdateAccountExpiry() {
        if let accountExpiryDate = accountExpiryDate {
            let accountExpiry = AccountExpiry(date: accountExpiryDate)

            if accountExpiry.isExpired {
                detailTitleLabel.text = NSLocalizedString(
                    "ACCOUNT_CELL_OUT_OF_TIME_LABEL",
                    tableName: "Settings",
                    comment: "Label displayed when user account ran out of time."
                )
                detailTitleLabel.textColor = .dangerColor
            } else {
                if let remainingTime = accountExpiry.formattedRemainingTime {
                    let localizedString = NSLocalizedString(
                        "ACCOUNT_CELL_TIME_LEFT_LABEL_FORMAT",
                        tableName: "Settings",
                        value: "%@ left",
                        comment: "The amount of time left on user account. Use %@ placeholder to position the localized text with the time duration left (i.e 10 days)."
                    )
                    let formattedString = String(format: localizedString, remainingTime)

                    detailTitleLabel.text = formattedString.uppercased()
                } else {
                    detailTitleLabel.text = ""
                }
                detailTitleLabel.textColor = UIColor.Cell.detailTextColor
            }
        } else {
            detailTitleLabel.text = ""
            detailTitleLabel.textColor = UIColor.Cell.detailTextColor
        }
    }

}
