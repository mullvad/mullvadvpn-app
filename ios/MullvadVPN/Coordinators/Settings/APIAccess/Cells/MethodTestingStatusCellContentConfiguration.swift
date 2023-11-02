//
//  MethodTestingStatusCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 27/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration for presenting the access method testing progress.
struct MethodTestingStatusCellContentConfiguration: UIContentConfiguration, Equatable {
    /// Sheet content configuration.
    var sheetConfiguration = AccessMethodActionSheetContentConfiguration()

    /// Layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.insetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return MethodTestingStatusCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
