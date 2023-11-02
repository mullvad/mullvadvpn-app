//
//  DynamicBackgroundConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Types providing dynamic background configuration based on cell configuration state.
protocol DynamicBackgroundConfiguration: UITableViewCell {
    typealias BackgroundConfigurationResolver = (UICellConfigurationState) -> UIBackgroundConfiguration

    /// Background configuration resolver closure.
    /// The closure is called immediately upon assignment, the returned configuration is assigned to `backgroundConfiguration`.
    /// All subsequent calls happen on `updateConfiguration(using:)`.
    var backgroundConfigurationResolver: BackgroundConfigurationResolver? { get set }
}

extension DynamicBackgroundConfiguration {
    /// Automatically maintains transparent background configuration in any cell state.
    func setAutoAdaptingClearBackgroundConfiguration() {
        backgroundConfigurationResolver = { _ in .clear() }
    }

    /// Automatically adjust background configuration for the cell state based on provided template and type of visual cell selection preference.
    ///
    /// - Parameters:
    ///   - backgroundConfiguration: a background configuration template.
    ///   - selectionType: a cell selection to apply.
    func setAutoAdaptingBackgroundConfiguration(
        _ backgroundConfiguration: UIBackgroundConfiguration,
        selectionType: UIBackgroundConfiguration.CellSelectionType
    ) {
        backgroundConfigurationResolver = { state in
            backgroundConfiguration.adapted(for: state, selectionType: selectionType)
        }
    }
}
