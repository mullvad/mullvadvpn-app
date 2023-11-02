//
//  AccessMethodActionSheetPresentationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Sheet presentation configuration.
struct AccessMethodActionSheetPresentationConfiguration: Equatable {
    /// Whether presentation dims background.
    /// When set to `false` the background is made transparent and all touches are passed through enabling interaction with the underlying view.
    var dimsBackground = true

    /// Whether presentation blurs the background behind the sheet pinned at the bottom.
    var blursSheetBackground = true

    /// Sheet configuration.
    var sheetConfiguration = AccessMethodActionSheetConfiguration()
}
