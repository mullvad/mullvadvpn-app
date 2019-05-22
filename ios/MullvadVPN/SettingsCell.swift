//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class SettingsCell: BasicTableViewCell {

    private let preferredMargins = UIEdgeInsets(top: 16, left: 28, bottom: 16, right: 12)

    override func awakeFromNib() {
        super.awakeFromNib()

        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor
    }

    override func layoutMarginsDidChange() {
        super.layoutMarginsDidChange()

        // enforce the preferred layout margins
        if contentView.layoutMargins != preferredMargins {
            contentView.layoutMargins = preferredMargins
        }
    }

}
