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
                detailTitleLabel.text = NSLocalizedString("OUT OF TIME", comment: "")
                detailTitleLabel.textColor = .dangerColor
            } else {
                if let remainingTime = accountExpiry.formattedRemainingTime {
                    let localizedString = NSLocalizedString("%@ left", comment: "")
                    let formattedString = String(format: localizedString, remainingTime)

                    detailTitleLabel.text = formattedString.uppercased()
                } else {
                    detailTitleLabel.text = ""
                }
                detailTitleLabel.textColor = .white
            }
        } else {
            detailTitleLabel.text = ""
            detailTitleLabel.textColor = .white
        }
    }

}
