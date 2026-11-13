import Combine
import MullvadTypes
// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import XCTest

@MainActor
final class ListAccessViewModelTests: XCTestCase {

    private var sut: ListAccessViewModel!
    private var interactor: MockListAccessMethodInteractor!

    override func setUp() async throws {
        try await super.setUp()

        interactor = MockListAccessMethodInteractor()
        sut = ListAccessViewModel(interactor: interactor)
    }

    override func tearDown() async throws {
        sut = nil
        interactor = nil

        try await super.tearDown()
    }

    // MARK: - Initial State

    func testInitialState() {
        XCTAssertTrue(sut.items.isEmpty)
    }

    // MARK: - Items Publisher

    func testItemsPublisherUpdatesItems() {
        // Arrange
        let expectedItems = ListAccessMethodItem.makeAccessMethodItems()

        // Act
        interactor.sendItems(expectedItems)

        // Assert
        let expectation = expectation(
            description: "Items are updated"
        )

        DispatchQueue.main.async {
            XCTAssertEqual(
                self.sut.items.count,
                expectedItems.count
            )
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
    }

    // MARK: - Default Cipher

    func testDefaultCipherReturnsInteractorDefaultCipher() {
        // Arrange
        let expectedCipher = "aes-256-gcm"
        interactor.defaultCipher = expectedCipher

        // Act
        let result = sut.defaultCipher

        // Assert
        XCTAssertEqual(result, expectedCipher)
    }

    // MARK: - Fetch

    func testFetchReturnsPersistentAccessMethodWhenFound() {
        // Arrange
        let persistentMethod = PersistentAccessMethod.makePersistentAccessMethod()
        let item = ListAccessMethodItem(
            id: persistentMethod.id,
            name: persistentMethod.name,
            detail: nil,
            isEnabled: persistentMethod.isEnabled,
            status: .off)

        interactor.accessMethods[item.id] = persistentMethod

        // Act
        let result = sut.fetch(item)

        // Assert
        XCTAssertNotNil(result)
    }

    func testFetchReturnsNilWhenAccessMethodIsNotFound() {
        // Arrange
        let persistentMethod = PersistentAccessMethod.makePersistentAccessMethod()
        let item = ListAccessMethodItem(
            id: persistentMethod.id,
            name: persistentMethod.name,
            detail: nil,
            isEnabled: persistentMethod.isEnabled,
            status: .off)

        // Act
        let result = sut.fetch(item)

        // Assert
        XCTAssertNil(result)
    }
}
