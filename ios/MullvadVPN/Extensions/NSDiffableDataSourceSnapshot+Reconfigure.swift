//
//  NSDiffableDataSourceSnapshot+Reconfigure.swift
//  MullvadVPN
//
//  Created by pronebird on 27/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSDiffableDataSourceSnapshot {
    /// Reconfigures cell on iOS 15 or newer, with fallback to reloading cells on earlier iOS.
    /// - Parameter itemIdentifiers: item identifiers to reconfigure when possible, otherwise reload.
    mutating func reconfigureOrReloadItems(_ itemIdentifiers: [ItemIdentifierType]) {
        if #available(iOS 15, *) {
            reconfigureItems(itemIdentifiers)
        } else {
            reloadItems(itemIdentifiers)
        }
    }
}
