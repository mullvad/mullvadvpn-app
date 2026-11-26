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
import MullvadMockData
import MullvadSettings
import MullvadTypes
import Testing

@Suite("EditAccessMethodInteractor Tests")
struct EditAccessMethodInteractorTests {
    private func makeSUT(
        methods: [PersistentAccessMethod] = [],
        directAccess: PersistentAccessMethod = .directMethod,
        shadowsocksCiphers: [String] = []
    ) -> (
        sut: EditAccessMethodInteractor,
        repository: AccessMethodRepositoryMock,
        proxyTester: ProxyConfigurationTesterMock
    ) {
        let repository = AccessMethodRepositoryMock(
            accessMethods: methods,
            directAccess: directAccess)
        repository.shadowsocksCiphers = shadowsocksCiphers

        let proxyTester = ProxyConfigurationTesterMock()

        let sut = EditAccessMethodInteractor(
            repository: repository,
            proxyConfigurationTester: proxyTester,
            breadcrumbsProvider: nil
        )

        return (
            sut,
            repository,
            proxyTester
        )
    }

    @Test("Last active access method cannot be disabled")
    func throwsWhenDisablingLastActiveAccessMethod() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, _, _) = makeSUT(methods: [method])
        #expect(throws: EditAccessMethodError.lastActiveMethod) {
            try sut.setEnabled(false, method: method)
        }
        #expect(sut.isLastActiveAccessMethod == true)
    }

    @Test("Fetches access method by ID")
    func fetchReturnsAccessMethod() throws {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, _, _) = makeSUT(methods: [method])

        let result = try sut.fetch(method.id)

        #expect(result == method)
    }

    @Test("Publishes updated access method when repository changes")
    func accessMethodPublisherPublishesUpdatedMethod() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, repository, _) = makeSUT(methods: [method])

        var receivedMethod: PersistentAccessMethod?

        let cancellable = sut.accessMethodPublisher(for: method.id)
            .sink { receivedMethod = $0 }

        var updatedMethod = method
        updatedMethod.isEnabled = false

        repository.setAccessMethods([updatedMethod])

        #expect(receivedMethod == updatedMethod)

        cancellable.cancel()
    }

    @Test("Saves access method and notifies API")
    func savePersistsAccessMethodAndNotifiesAPI() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, repository, _) = makeSUT(methods: [method])

        var updatedMethod = method
        updatedMethod.name = UUID().uuidString

        sut.save(updatedMethod)

        #expect(repository.savedAccessMethod == updatedMethod)
        #expect(repository.savedNotifyingAPI == true)
    }

    @Test("Disables access method when at least another method is active")
    func disablesAccessMethod() throws {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let anotherMethod = PersistentAccessMethod.makePersistentAccessMethod()

        let (sut, repository, _) = makeSUT(methods: [method, anotherMethod])

        try sut.setEnabled(false, method: method)

        #expect(repository.fetch(by: method.id)?.isEnabled == false)
    }

    @Test("Deletes access method from repository")
    func deleteRemovesAccessMethod() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let anotherMethod = PersistentAccessMethod.makePersistentAccessMethod()

        let (sut, repository, _) = makeSUT(methods: [method, anotherMethod])

        sut.delete(method)

        #expect(repository.fetch(by: method.id) == nil)
        #expect(repository.fetch(by: anotherMethod.id) == anotherMethod)
    }

    @Test("Enables direct access when deleting the last enabled method")
    func deleteEnablesDirectAccessWhenNoMethodsAreEnabled() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let directAccess = PersistentAccessMethod.directMethod

        let (sut, repository, _) = makeSUT(
            methods: [method],
            directAccess: directAccess
        )

        sut.delete(method)

        #expect(repository.fetch(by: method.id) == nil)
        #expect(repository.fetch(by: directAccess.id) == directAccess)
    }

    @Test("Throws notFound when access method does not exist")
    func fetchThrowsWhenAccessMethodDoesNotExist() {
        let (sut, _, _) = makeSUT()

        #expect(throws: EditAccessMethodError.notFound) {
            try sut.fetch(UUID())
        }
    }

    @Test("Starts proxy configuration testing")
    func testStartsProxyConfigurationTesting() async throws {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, _, proxyTester) = makeSUT()

        try await sut.test(method)

        let testedConfigurations = await proxyTester.testedConfigurations

        #expect(testedConfigurations == [method])
    }

    @Test("Cancels ongoing proxy configuration testing")
    func cancelTestingCancelsProxyTester() async throws {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, _, proxyTester) = makeSUT()

        try await sut.test(method)
        await sut.cancelTesting()

        let cancelCallCount = await proxyTester.cancelCallCount

        #expect(cancelCallCount == 1)
    }
}
