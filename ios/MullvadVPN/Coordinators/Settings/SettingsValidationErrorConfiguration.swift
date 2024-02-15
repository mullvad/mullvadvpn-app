//
//  SettingsValidationErrorConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct SettingsValidationErrorConfiguration: UIContentConfiguration {
    var errors: [SettingsValidationErrorProtocol] = []
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.settingsValidationErrorLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return SettingsValidationErrorContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension SettingsValidationErrorConfiguration: Equatable {
    static func == (lhs: SettingsValidationErrorConfiguration, rhs: SettingsValidationErrorConfiguration) -> Bool {
        lhs.errors.map { $0.errorDescription } == rhs.errors.map { $0.errorDescription }
    }
}
