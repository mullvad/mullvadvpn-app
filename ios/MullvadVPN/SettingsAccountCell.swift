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

    override func awakeFromNib() {
        super.awakeFromNib()

        // Remove the right margin since the accessory view adds it automatically
        contentView.layoutMargins.right = 0
    }

}
