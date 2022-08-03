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
        guard let accountExpiryDate = accountExpiryDate else {
            detailTitleLabel.text = ""
            detailTitleLabel.textColor = UIColor.Cell.detailTextColor
            return
        }

        guard accountExpiryDate > Date() else {
            detailTitleLabel.text = NSLocalizedString(
                "ACCOUNT_CELL_OUT_OF_TIME_LABEL",
                tableName: "Settings",
                value: "OUT OF TIME",
                comment: ""
            )
            detailTitleLabel.textColor = .dangerColor
            return
        }

        let formattedTime = CustomDateComponentsFormatting.localizedString(
            from: Date(),
            to: accountExpiryDate,
            unitsStyle: .full
        )

        detailTitleLabel.text = formattedTime.map { remainingTimeString in
            let localizedString = NSLocalizedString(
                "ACCOUNT_CELL_TIME_LEFT_LABEL_FORMAT",
                tableName: "Settings",
                value: "%@ left",
                comment: ""
            )

            return String(format: localizedString, remainingTimeString).uppercased()
        } ?? ""
        detailTitleLabel.textColor = UIColor.Cell.detailTextColor
    }
}
