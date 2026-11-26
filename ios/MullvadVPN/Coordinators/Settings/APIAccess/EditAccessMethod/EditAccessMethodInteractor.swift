// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

enum EditAccessMethodError: Error {
    case notFound
    case lastActiveMethod
}

protocol EditAccessMethodInteractorProtocol: Sendable {
    var isLastActiveAccessMethod: Bool { get }
    var hasMethodsWithInvalidCiphers: Bool { get }
    var shadowsocksCiphers: [String] { get }

    func accessMethodPublisher(for id: UUID) -> AnyPublisher<PersistentAccessMethod?, Never>

    func isLastActiveAccessMethod(_ method: PersistentAccessMethod) -> Bool

    func fetch(_ id: UUID) throws -> PersistentAccessMethod
    func save(_ method: PersistentAccessMethod)
    func setEnabled(_ isEnabled: Bool, method: PersistentAccessMethod) throws
    func delete(_ method: PersistentAccessMethod)

    func test(_ method: PersistentAccessMethod) async throws
    func cancelTesting() async
}

final class EditAccessMethodInteractor: EditAccessMethodInteractorProtocol, @unchecked Sendable {
    let repository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    let breadcrumbsProvider: BreadcrumbsProvider?

    private var list: [PersistentAccessMethod] = []
    private var cancellables = Set<Combine.AnyCancellable>()

    init(
        repository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        breadcrumbsProvider: BreadcrumbsProvider?
    ) {
        self.repository = repository
        self.proxyConfigurationTester = proxyConfigurationTester
        self.breadcrumbsProvider = breadcrumbsProvider

        repository.accessMethodsPublisher
            .sink { updatedList in
                self.list = updatedList
            }
            .store(in: &cancellables)
    }

    var isLastActiveAccessMethod: Bool {
        list.filter(\.isEnabled).count == 1
    }

    var hasMethodsWithInvalidCiphers: Bool {
        let ciphers = repository.shadowsocksCiphers

        return list.contains { accessMethod in
            guard case .shadowsocks(let config) = accessMethod.proxyConfiguration else {
                return false
            }

            return !ciphers.contains(config.cipher)
        }
    }

    var shadowsocksCiphers: [String] {
        repository.shadowsocksCiphers
    }

    func isLastActiveAccessMethod(_ method: PersistentAccessMethod) -> Bool {
        let activeList = self.list.filter(\.isEnabled)
        return activeList.count == 1 && activeList.first == method
    }

    func accessMethodPublisher(for id: UUID) -> AnyPublisher<PersistentAccessMethod?, Never> {
        repository.accessMethodsPublisher
            .map { methods in
                methods.first { $0.id == id }
            }
            .eraseToAnyPublisher()
    }

    func fetch(_ id: UUID) throws -> PersistentAccessMethod {
        guard let accessMethod = list.first(where: { $0.id == id }) else {
            throw EditAccessMethodError.notFound
        }

        return accessMethod
    }

    func save(_ method: PersistentAccessMethod) {
        repository.save(method, notifyingAPI: true)
        guard !hasMethodsWithInvalidCiphers else {
            return
        }

        Task { @MainActor in
            breadcrumbsProvider?.remove(breadcrumb: .warning(.apiAccess))
        }
    }

    func setEnabled(_ isEnabled: Bool, method: PersistentAccessMethod) throws {
        guard isEnabled || !isLastActiveAccessMethod(method) else {
            throw EditAccessMethodError.lastActiveMethod
        }
        var updatedMethod = method
        updatedMethod.isEnabled = isEnabled
        save(updatedMethod)
    }

    func delete(_ method: PersistentAccessMethod) {
        repository.delete(id: method.id)

        // Enable direct access if all methods are disabled
        if !list.contains(where: \.isEnabled) {
            save(repository.directAccess)
        }
    }

    func test(_ method: PersistentAccessMethod) async throws {
        try await proxyConfigurationTester.start(configuration: method)
    }

    func cancelTesting() async {
        await proxyConfigurationTester.cancel()
    }
}
