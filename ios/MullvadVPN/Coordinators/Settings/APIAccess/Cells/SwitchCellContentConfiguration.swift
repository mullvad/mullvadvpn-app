//
//  SwitchCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration presenting a label and switch control.
struct SwitchCellContentConfiguration: UIContentConfiguration, Equatable {
    var text: String?
    var isOn = false
    var onChange: UIAction?

    var textProperties = TextProperties()

    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.insetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return SwitchCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension SwitchCellContentConfiguration {
    struct TextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.titleTextColor
    }
}
