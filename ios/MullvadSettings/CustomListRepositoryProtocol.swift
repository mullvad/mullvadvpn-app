//
//  CustomListRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadTypes
public protocol CustomListRepositoryProtocol {
    /// Save a custom list. If the list doesn't already exist, it must have a unique name.
    /// - Parameter list: a custom list.
    func save(list: CustomList) throws

    /// Delete custom list by id.
    /// - Parameter id: an access method id.
    func delete(id: UUID)

    /// Fetch custom list  by id.
    /// - Parameter id: a custom list id.
    /// - Returns: a persistent custom list model upon success, otherwise `nil`.
    func fetch(by id: UUID) -> CustomList?

    /// Fetch all  custom list.
    /// - Returns: all custom list model .
    func fetchAll() -> [CustomList]
}
