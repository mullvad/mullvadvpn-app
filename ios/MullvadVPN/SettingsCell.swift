//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class SettingsCell: BasicTableViewCell {

    private let preferredMargins = UIEdgeInsets(top: 36, left: 24, bottom: 36, right: 12)

    override func awakeFromNib() {
        super.awakeFromNib()

        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor

        layoutMargins = preferredMargins
    }

}
