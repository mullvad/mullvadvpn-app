//
//  SettingsValidationErrorConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct SettingsValidationErrorConfiguration: UIContentConfiguration, Equatable {
    var errors: [CustomListFieldValidationError] = []
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.settingsValidationErrorLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return SettingsValidationErrorContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
