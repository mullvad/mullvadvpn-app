//
//  CheckableSettingsCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CheckableSettingsCell: SettingsCell {
    let checkboxView = CheckboxView()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        setLeftView(checkboxView, spacing: UIMetrics.SettingsCell.checkableSettingsCellLeftViewSpacing)
        selectedBackgroundView?.backgroundColor = .clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setLeftView(checkboxView, spacing: UIMetrics.SettingsCell.checkableSettingsCellLeftViewSpacing)
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        checkboxView.isChecked = selected
    }

    override func applySubCellStyling() {
        super.applySubCellStyling()

        contentView.layoutMargins.left = 0
    }
}
