//
//  ListAccessMethodItem.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A concrete implementation of an API access list item.
struct ListAccessMethodItem: Hashable, Identifiable, Equatable {
    let id: UUID

    /// The localized name of an API method.
    let name: String

    /// The detailed information displayed alongside.
    let detail: String?
}
