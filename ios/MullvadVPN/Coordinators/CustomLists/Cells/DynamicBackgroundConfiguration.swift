// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
