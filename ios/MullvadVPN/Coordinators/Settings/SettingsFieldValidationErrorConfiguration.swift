//
//  SettingsFieldValidationErrorConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct SettingsFieldValidationError: LocalizedError, Equatable {
    var errorDescription: String?
}

struct SettingsFieldValidationErrorConfiguration: UIContentConfiguration, Equatable {
    var errors: [SettingsFieldValidationError] = []
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.settingsValidationErrorLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return SettingsFieldValidationErrorContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
