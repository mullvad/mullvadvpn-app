//
//  AccessMethodActionSheetConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The context in which the sheet is being used.
enum AccessMethodActionSheetContext: Equatable {
    /// The variant describing the context when adding a new method.
    ///
    /// In this context, the sheet offers user to add access method anyway or cancel, once the API tests indicate a failure.
    /// (See `contentConfiguration.status`)
    case addNew

    /// The variant describing the context when the existing API method is being tested or edited as a part of proxy configuration sub-navigation.
    ///
    /// In this context, the sheet only offers user to cancel testing the access method.
    case proxyConfiguration
}

/// The sheet configuration.
struct AccessMethodActionSheetConfiguration: Equatable {
    /// The sheet presentation context.
    var context: AccessMethodActionSheetContext = .addNew

    /// The sheet content configuration.
    var contentConfiguration = AccessMethodActionSheetContentConfiguration()
}
