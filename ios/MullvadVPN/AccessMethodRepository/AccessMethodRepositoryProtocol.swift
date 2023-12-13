//
//  AccessMethodRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

protocol AccessMethodRepositoryProtocol {
    /// Publisher that propagates a snapshot of persistent store upon modifications.
    var publisher: PassthroughSubject<[PersistentAccessMethod], Never> { get }

    /// Add new access method.
    /// - Parameter method: persistent access method model.
    func add(_ method: PersistentAccessMethod)

    /// Update access method locating existing entry by id.
    /// - Parameter method: persistent access method model.
    func update(_ method: PersistentAccessMethod)

    /// Delete access method by id.
    /// - Parameter id: an access method id.
    func delete(id: UUID)

    /// Fetch access method by id.
    /// - Parameter id: an access method id.
    /// - Returns: a persistent access method model upon success, otherwise `nil`.
    func fetch(by id: UUID) -> PersistentAccessMethod?

    /// Fetch all access methods from the persistent store.
    /// - Returns: an array of all persistent access method.
    func fetchAll() -> [PersistentAccessMethod]
}
