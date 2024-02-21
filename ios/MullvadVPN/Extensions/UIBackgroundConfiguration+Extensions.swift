//
//  UIBackgroundConfiguration+Extensions.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIBackgroundConfiguration {
    /// Type of cell selection used in Mullvad UI.
    enum CellSelectionType {
        /// Dimmed blue.
        case dimmed
        /// Bright green.
        case green
    }

    /// Returns a plain cell background configuration adapted for Mullvad UI.
    /// - Returns: a background configuration
    static func mullvadListPlainCell() -> UIBackgroundConfiguration {
        var config = listPlainCell()
        config.backgroundColor = UIColor.Cell.Background.normal
        return config
    }

    /// Returns the corresponding grouped cell background configuration adapted for Mullvad UI.
    /// - Returns: a background configuration
    static func mullvadListGroupedCell() -> UIBackgroundConfiguration {
        var config = listGroupedCell()
        config.backgroundColor = UIColor.Cell.Background.normal
        return config
    }

    /// Adapt background configuration for the cell state and selection type.
    ///
    /// - Parameters:
    ///   - state: a cell state.
    ///   - selectionType: a desired selecton type.
    /// - Returns: new background configuration.
    func adapted(
        for state: UICellConfigurationState,
        selectionType: CellSelectionType
    ) -> UIBackgroundConfiguration {
        var config = self
        config.backgroundColor = state.mullvadCellBackgroundColor(selectionType: selectionType)
        return config
    }
}

extension UICellConfigurationState {
    /// Produce background color for the given state and cell selection type.
    ///
    /// - Parameter selectionType: cell selection type.
    /// - Returns: a background color to apply to cell.
    func mullvadCellBackgroundColor(selectionType: UIBackgroundConfiguration.CellSelectionType) -> UIColor {
        switch selectionType {
        case .dimmed:
            if isSelected || isHighlighted {
                UIColor.Cell.Background.selectedAlt
            } else if isDisabled {
                UIColor.Cell.Background.disabled
            } else {
                UIColor.Cell.Background.normal
            }

        case .green:
            if isSelected || isHighlighted {
                UIColor.Cell.Background.selected
            } else if isDisabled {
                UIColor.Cell.Background.disabled
            } else {
                UIColor.Cell.Background.normal
            }
        }
    }
}
