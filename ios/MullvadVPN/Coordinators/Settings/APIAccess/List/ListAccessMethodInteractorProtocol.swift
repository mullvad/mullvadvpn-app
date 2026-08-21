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
import MullvadSettings
import MullvadTypes

/// Types describing API access list interactor.
protocol ListAccessMethodInteractorProtocol {
    /// Publisher that produces a list of method items upon persistent store modifications.
    var itemsPublisher: AnyPublisher<[ListAccessMethodItem], Never> { get }

    /// Publisher that produces the last reachable method item upon persistent store modifications.
    var itemInUsePublisher: AnyPublisher<ListAccessMethodItem?, Never> { get }

    /// Available Shadowsocks ciphers.
    var shadowsocksCiphers: [String] { get }

    /// Returns an item by id.
    func item(by id: UUID) -> ListAccessMethodItem?

    /// Fetch all items.
    func fetch() -> [ListAccessMethodItem]

    /// Returns an item by id.
    func accessMethod(by id: UUID) -> PersistentAccessMethod?
}
