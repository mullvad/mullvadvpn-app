//
//  AccessMethodViewModel+NavigationItem.swift
//  MullvadVPN
//
//  Created by pronebird on 29/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension AccessMethodViewModel {
    /// Title suitable for navigation item.
    /// User-defined name is preferred unless it's blank, in which case the name of access method is used instead.
    var navigationItemTitle: String {
        if name.trimmingCharacters(in: .whitespaces).isEmpty {
            method.localizedDescription
        } else {
            name
        }
    }
}
