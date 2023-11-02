//
//  AccessMethodActionSheetDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Sheet container view delegate.
protocol AccessMethodActionSheetDelegate: AnyObject {
    /// User tapped the cancel button.
    func sheetDidCancel(_ sheet: AccessMethodActionSheetContainerView)

    /// User tapped the add button.
    func sheetDidAdd(_ sheet: AccessMethodActionSheetContainerView)
}
