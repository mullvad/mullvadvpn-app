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
import Testing

@MainActor
@Suite("ListAccessViewModel Tests")
struct ListAccessViewModelTests {

    private func makeSUT() -> (
        sut: ListAccessViewModel,
        interactor: MockListAccessMethodInteractor
    ) {
        let interactor = MockListAccessMethodInteractor()
        let sut = ListAccessViewModel(interactor: interactor)

        return (sut, interactor)
    }

    // MARK: - Items Publisher

    @Test("Items are updated when the publisher emits values")
    func itemsPublisherUpdatesItems() async {
        let (sut, interactor) = makeSUT()

        let expectedItems = ListAccessMethodItem.makeAccessMethodItems()

        interactor.sendItems(expectedItems)

        #expect(sut.items.count == expectedItems.count)
    }

    // MARK: - Default Cipher

    @Test("Default cipher returns the interactor default cipher")
    func defaultCipherReturnsInteractorDefaultCipher() {
        let (sut, interactor) = makeSUT()

        let expectedCipher = "aes-256-gcm"
        interactor.defaultCipher = expectedCipher

        let result = sut.defaultCipher

        #expect(result == expectedCipher)
    }

    // MARK: - Fetch

    @Test("Fetch returns persistent access method when found")
    func fetchReturnsPersistentAccessMethodWhenFound() {
        let (sut, interactor) = makeSUT()

        let persistentMethod =
            PersistentAccessMethod.makePersistentAccessMethod()

        let item = ListAccessMethodItem(
            id: persistentMethod.id,
            name: persistentMethod.name,
            detail: nil,
            isEnabled: persistentMethod.isEnabled,
            status: .off
        )

        interactor.accessMethods[item.id] = persistentMethod

        let result = sut.fetch(item)

        #expect(result != nil)
    }

    @Test("Fetch returns nil when access method is not found")
    func fetchReturnsNilWhenAccessMethodIsNotFound() {
        let (sut, _) = makeSUT()

        let persistentMethod = PersistentAccessMethod.makePersistentAccessMethod()

        let item = ListAccessMethodItem(
            id: persistentMethod.id,
            name: persistentMethod.name,
            detail: nil,
            isEnabled: persistentMethod.isEnabled,
            status: .off
        )

        let result = sut.fetch(item)

        #expect(result == nil)
    }
}
