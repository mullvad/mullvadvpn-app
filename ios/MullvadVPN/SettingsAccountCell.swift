//
//  SettingsAccountCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class SettingsAccountCell: SettingsCell {

    @IBOutlet var titleLabel: UILabel!
    @IBOutlet var expiryLabel: UILabel!

    var accountExpiryDate: Date? {
        didSet {
            didUpdateAccountExpiry()
        }
    }

    private func didUpdateAccountExpiry() {
        if let accountExpiryDate = accountExpiryDate {
            let accountExpiry = AccountExpiry(date: accountExpiryDate)

            if accountExpiry.isExpired {
                expiryLabel.text = NSLocalizedString("OUT OF TIME", tableName: "Settings", comment: "")
                expiryLabel.textColor = .dangerColor
            } else {
                let remainingTime = accountExpiry.formattedRemainingTime
                let localizedString = NSLocalizedString("%@ left", tableName: "Settings", comment: "")
                let formattedString = String(format: localizedString, remainingTime)

                expiryLabel.text = formattedString.uppercased()
                expiryLabel.textColor = .white
            }
        } else {
            expiryLabel.text = ""
            expiryLabel.textColor = .white
        }
    }

}
