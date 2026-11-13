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
import Foundation
import MullvadTypes

final class MockListAccessMethodInteractor: ListAccessMethodInteractorProtocol {

    private let itemsSubject = PassthroughSubject<[ListAccessMethodItem], Never>()

    var itemsPublisher: AnyPublisher<[ListAccessMethodItem], Never> {
        itemsSubject.eraseToAnyPublisher()
    }

    var shadowsocksCiphers: [String] = [
        "aes-128-gcm",
        "aes-256-gcm",
        "chacha20-ietf-poly1305",
    ]

    var defaultCipher: String = "aes-256-gcm"

    var accessMethods: [UUID: PersistentAccessMethod] = [:]

    func accessMethod(by id: UUID) -> PersistentAccessMethod? {
        accessMethods[id]
    }

    // MARK: - Test Helpers

    func sendItems(_ items: [ListAccessMethodItem]) {
        itemsSubject.send(items)
    }
}
