// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadTypes

public protocol AccessMethodRepositoryDataSource: Sendable {
    /// Available Shadowsocks ciphers.
    var shadowsocksCiphers: [String] { get }

    /// Publisher that propagates a snapshot of all access methods upon modifications.
    var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> { get }

    /// - Returns: the default strategy.
    var directAccess: PersistentAccessMethod { get }

    /// Fetch all access method from the persistent store.
    /// - Returns: an array of all persistent access method.
    func fetchAll() -> [PersistentAccessMethod]

    /// Save last reachable access method to the persistent store.
    func requestAccessMethod(_ method: PersistentAccessMethod)

    /// Fetch last reachable access method from the persistent store.
    func fetchLastReachable() -> PersistentAccessMethod
}

public protocol AccessMethodRepositoryProtocol: AccessMethodRepositoryDataSource {
    /// Publisher that propagates a snapshot of last reachable access method upon modifications.
    var currentAccessMethodPublisher: AnyPublisher<PersistentAccessMethod, Never> { get }

    /// Add new access method.
    /// - Parameter method: persistent access method model.
    func save(_ method: PersistentAccessMethod, notifyingAPI: Bool)

    /// Delete access method by id.
    /// - Parameter id: an access method id.
    func delete(id: UUID)

    /// Fetch access method by id.
    /// - Parameter id: an access method id.
    /// - Returns: a persistent access method model upon success, otherwise `nil`.
    func fetch(by id: UUID) -> PersistentAccessMethod?

    ///  Refreshes the storage with default values.
    func addDefaultsMethods()
}
