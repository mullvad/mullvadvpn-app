//
//  SettingsDNSServerAddressCell.swift
//  MullvadVPN
//
//  Created by pronebird on 08/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsDNSServerAddressCell: SettingsCell {
    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView?.backgroundColor = UIColor.SubCell.backgroundColor

        setLayoutMargins()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()
        
        setLayoutMargins()
    }

    private func setLayoutMargins() {
        var contentMargins = UIMetrics.settingsCellLayoutMargins
        contentMargins.left += UIMetrics.cellIndentationWidth

        contentView.layoutMargins = contentMargins
    }
}
