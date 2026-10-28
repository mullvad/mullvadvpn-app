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
    case cannotDisableLastEnabledMethod
    case unreachable
}

protocol EditAccessMethodInteractorProtocol {

    var hasMethodsWithInvalidCiphers: Bool { get }
    var shadowsocksCiphers: [String] { get }

    func fetch(_ id: UUID) throws -> PersistentAccessMethod
    func save(_ method: PersistentAccessMethod)
    func setEnabled(
        _ isEnabled: Bool,
        for method: PersistentAccessMethod
    ) -> Result<PersistentAccessMethod, EditAccessMethodError>
    func delete(_ method: PersistentAccessMethod)

    func startTesting(
        _ method: PersistentAccessMethod,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    )
    func stopTesting()
}

final class EditAccessMethodInteractor: EditAccessMethodInteractorProtocol {
    let repository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    let breadcrumbsProvider: BreadcrumbsProvider

    private var list: [PersistentAccessMethod] = []
    private var cancellables = Set<Combine.AnyCancellable>()

    init(
        repository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        breadcrumbsProvider: BreadcrumbsProvider
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
        breadcrumbsProvider.remove(breadcrumb: .warning(.apiAccess))
    }

    func setEnabled(
        _ isEnabled: Bool,
        for method: PersistentAccessMethod
    ) -> Result<PersistentAccessMethod, EditAccessMethodError> {
        if !isEnabled && list.filter(\.isEnabled).count <= 1 {
            return .failure(.cannotDisableLastEnabledMethod)
        }

        var updatedMethod = method
        updatedMethod.isEnabled = isEnabled
        save(updatedMethod)
        return .success(updatedMethod)
    }

    func delete(_ method: PersistentAccessMethod) {
        repository.delete(id: method.id)

        // Enable direct access if all methods are disabled
        if !list.contains(where: \.isEnabled) {
            _ = setEnabled(true, for: repository.directAccess)
        }
    }

    func startTesting(
        _ method: PersistentAccessMethod,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    ) {
        proxyConfigurationTester.start(configuration: method) { error in
            if let error {
                completion(.failure(error))
            } else {
                completion(.success(()))
            }
        }
    }

    func stopTesting() {
        proxyConfigurationTester.cancel()
    }

}
