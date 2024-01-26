//
//  AccessMethodRepositoryProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine

public protocol AccessMethodRepositoryDataSource {
    /// Publisher that propagates a snapshot of all access methods upon modifications.
    var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> { get }

    /// - Returns: the default strategy.
    var directAccess: PersistentAccessMethod { get }

    /// Fetch all access method from the persistent store.
    /// - Returns: an array of all persistent access method.
    func fetchAll() -> [PersistentAccessMethod]

    /// Save last reachable access method to the persistent store.
    func saveLastReachable(_ method: PersistentAccessMethod)

    /// Fetch last reachable access method from the persistent store.
    func fetchLastReachable() -> PersistentAccessMethod
}

public protocol AccessMethodRepositoryProtocol: AccessMethodRepositoryDataSource {
    /// Publisher that propagates a snapshot of last reachable access method upon modifications.
    var lastReachableAccessMethodPublisher: AnyPublisher<PersistentAccessMethod, Never> { get }

    /// Add new access method.
    /// - Parameter method: persistent access method model.
    func save(_ method: PersistentAccessMethod)

    /// Delete access method by id.
    /// - Parameter id: an access method id.
    func delete(id: UUID)

    /// Fetch access method by id.
    /// - Parameter id: an access method id.
    /// - Returns: a persistent access method model upon success, otherwise `nil`.
    func fetch(by id: UUID) -> PersistentAccessMethod?

    ///  Refreshes the storage with default values.
    func reloadWithDefaultsAfterDataRemoval()
}
