//
//  AccessMethodActionSheetPresentationDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Sheet presentation delegate.
protocol AccessMethodActionSheetPresentationDelegate: AnyObject {
    /// User tapped the cancel button.
    func sheetDidCancel(sheetPresentation: AccessMethodActionSheetPresentation)

    /// User tapped the add button.
    func sheetDidAdd(sheetPresentation: AccessMethodActionSheetPresentation)
}
