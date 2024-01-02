//
//  MethodSettingsValidationErrorContentConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration for presenting the access method testing progress.
struct MethodSettingsValidationErrorContentConfiguration: UIContentConfiguration, Equatable {
    /// Field validation errors.
    var fieldErrors: [AccessMethodFieldValidationError] = []

    /// Layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return MethodSettingsValidationErrorContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
