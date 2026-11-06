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

/// A concrete implementation of an API access list interactor.
struct ListAccessMethodInteractor: ListAccessMethodInteractorProtocol {
    let repository: AccessMethodRepositoryProtocol

    init(repository: AccessMethodRepositoryProtocol) {
        self.repository = repository
    }

    var itemsPublisher: AnyPublisher<[ListAccessMethodItem], Never> {
        Publishers.CombineLatest(repository.accessMethodsPublisher, repository.currentAccessMethodPublisher)
            .receive(on: RunLoop.main)
            .map { methods, currentMethod in
                methods.map { method in
                    let cipherIsValid =
                        if case let .shadowsocks(configuration) = method.proxyConfiguration {
                            shadowsocksCiphers.contains(configuration.cipher)
                        } else {
                            true
                        }
                    return method.toListItem(
                        isInUse: method.id == currentMethod.id, configurationIsValid: cipherIsValid)
                }
            }.eraseToAnyPublisher()

    }

    var shadowsocksCiphers: [String] {
        repository.shadowsocksCiphers
    }

    func accessMethod(by id: UUID) -> PersistentAccessMethod? {
        repository.fetch(by: id)
    }
}

extension PersistentAccessMethod {
    fileprivate func toListItem(
        isInUse: Bool,
        configurationIsValid: Bool
    ) -> ListAccessMethodItem {
        let sanitizedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
        let itemName = sanitizedName.isEmpty ? kind.localizedDescription : sanitizedName

        var status: AccessMethodStatus = isEnabled ? .on : .off

        if !configurationIsValid {
            status = .warning("Unsupported cipher")
        } else if isInUse {
            status = .inUse
        }

        return ListAccessMethodItem(
            id: id,
            name: itemName,
            detail: kind.localizedDescription,
            isEnabled: isEnabled,
            status: status
        )
    }
}
