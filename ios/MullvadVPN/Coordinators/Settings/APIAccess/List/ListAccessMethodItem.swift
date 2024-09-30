//
//  ListAccessMethodItem.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A concrete implementation of an API access list item.
struct ListAccessMethodItem: Hashable, Identifiable, Equatable {
    /// The unique ID.
    let id: UUID

    /// The localized name.
    let name: String

    /// The detailed information displayed alongside.
    let detail: String?

    /// Whether method is enabled or not.
    let isEnabled: Bool
}
