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

@Suite("ListAccessMethodInteractor Tests")
struct ListAccessMethodInteractorTests {

    private func makeSUT() -> MockListAccessMethodInteractor {
        MockListAccessMethodInteractor()
    }

    // MARK: - Shadowsocks Ciphers

    @Test("Returns available Shadowsocks ciphers")
    func shadowsocksCiphersReturnsAvailableCiphers() {
        let sut = makeSUT()

        // Arrange
        let expectedCiphers = [
            "aes-128-gcm",
            "aes-256-gcm",
            "chacha20-ietf-poly1305",
        ]

        sut.shadowsocksCiphers = expectedCiphers

        // Act
        let result = sut.shadowsocksCiphers

        // Assert
        #expect(result == expectedCiphers)
    }

    // MARK: - Default Cipher

    @Test("Returns the expected default cipher")
    func defaultCipherReturnsExpectedCipher() {
        let sut = makeSUT()

        // Arrange
        let expectedCipher = "aes-256-gcm"
        sut.defaultCipher = expectedCipher

        // Act
        let result = sut.defaultCipher

        // Assert
        #expect(result == expectedCipher)
    }

    // MARK: - Access Method Lookup

    @Test("Returns access method for an existing ID")
    func accessMethodReturnsMethodForExistingID() {
        let sut = makeSUT()

        // Arrange
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        sut.accessMethods[method.id] = method

        // Act
        let result = sut.accessMethod(by: method.id)

        // Assert
        #expect(result != nil)
        #expect(result == method)
    }

    @Test("Returns nil for an unknown ID")
    func accessMethodReturnsNilForUnknownID() {
        let sut = makeSUT()

        // Arrange
        let unknownID = UUID()

        // Act
        let result = sut.accessMethod(by: unknownID)

        // Assert
        #expect(result == nil)
    }

    // MARK: - Items Publisher

    @Test("Items publisher emits updated items")
    func itemsPublisherEmitsUpdatedItems() async {
        let sut = makeSUT()

        // Arrange
        let expectedItems = ListAccessMethodItem.makeAccessMethodItems()

        var receivedItems: [ListAccessMethodItem] = []

        let cancellable = sut.itemsPublisher
            .sink { items in
                receivedItems = items
            }

        defer {
            cancellable.cancel()
        }

        // Act
        sut.sendItems(expectedItems)

        // Assert
        #expect(receivedItems.count == expectedItems.count)
    }
}
