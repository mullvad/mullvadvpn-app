//
//  SettingsSwitchCell.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsSwitchCell: SettingsCell {

    let switchContainer = CustomSwitchContainer()
    var switchControl: CustomSwitch {
        return switchContainer.control
    }

    var action: ((Bool) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        self.accessoryView = switchContainer

        switchControl.addTarget(self, action: #selector(switchValueDidChange), for: .valueChanged)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func switchValueDidChange() {
        self.action?(self.switchControl.isOn)
    }

}
