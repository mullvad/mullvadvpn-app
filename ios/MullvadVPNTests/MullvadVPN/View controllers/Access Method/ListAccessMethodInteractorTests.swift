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
import XCTest

final class ListAccessMethodInteractorTests: XCTestCase {

    private var sut: MockListAccessMethodInteractor!
    private var cancellables: Set<Combine.AnyCancellable>!

    override func setUp() {
        super.setUp()

        sut = MockListAccessMethodInteractor()
        cancellables = []
    }

    override func tearDown() {
        cancellables = nil
        sut = nil

        super.tearDown()
    }

    // MARK: - Shadowsocks Ciphers

    func testShadowsocksCiphersReturnsAvailableCiphers() {
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
        XCTAssertEqual(result, expectedCiphers)
    }

    // MARK: - Default Cipher

    func testDefaultCipherReturnsExpectedCipher() {
        // Arrange
        let expectedCipher = "aes-256-gcm"
        sut.defaultCipher = expectedCipher

        // Act
        let result = sut.defaultCipher

        // Assert
        XCTAssertEqual(result, expectedCipher)
    }

    // MARK: - Access Method Lookup

    func testAccessMethodReturnsMethodForExistingID() {
        // Arrange
        let method = PersistentAccessMethod.makePersistentAccessMethod()

        sut.accessMethods[method.id] = method

        // Act
        let result = sut.accessMethod(by: method.id)

        // Assert
        XCTAssertNotNil(result)
        XCTAssertTrue(result == method)
    }

    func testAccessMethodReturnsNilForUnknownID() {
        // Arrange
        let unknownID = UUID()

        // Act
        let result = sut.accessMethod(by: unknownID)

        // Assert
        XCTAssertNil(result)
    }

    // MARK: - Items Publisher

    func testItemsPublisherEmitsUpdatedItems() {
        // Arrange
        let expectation = expectation(
            description: "Items publisher emits updated items"
        )

        let expectedItems = ListAccessMethodItem.makeAccessMethodItems()

        var receivedItems: [ListAccessMethodItem]?

        sut.itemsPublisher
            .sink { items in
                receivedItems = items
                expectation.fulfill()
            }
            .store(in: &cancellables)

        // Act
        sut.sendItems(expectedItems)

        // Assert
        wait(for: [expectation], timeout: 1.0)

        XCTAssertEqual(receivedItems?.count, expectedItems.count)
    }
}
