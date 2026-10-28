// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol ListAccessMethodViewFactoryProtocol {
    func makeListViewModel() -> any ListAccessViewModelProtocol
    func makeAddMethodViewModel() -> any ConfigureApiAccessMethodViewModelProtocol
    func makeConfigureAccessMethodViewModel(for item: PersistentAccessMethod)
        -> any ConfigureApiAccessMethodViewModelProtocol
    func makeEditMethodViewModel(for item: PersistentAccessMethod) -> any EditAccessMethodViewModelProtocol
}

@MainActor
struct ListAccessMethodViewFactory: ListAccessMethodViewFactoryProtocol {
    private let repository: AccessMethodRepositoryProtocol
    private let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    private let breadcrumbsProvider: BreadcrumbsProvider

    init(
        repository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        breadcrumbsProvider: BreadcrumbsProvider
    ) {
        self.repository = repository
        self.proxyConfigurationTester = proxyConfigurationTester
        self.breadcrumbsProvider = breadcrumbsProvider
    }

    func makeListViewModel() -> any ListAccessViewModelProtocol {
        ListAccessViewModel(
            interactor: ListAccessMethodInteractor(
                repository: repository
            )
        )
    }

    func makeAddMethodViewModel() -> any ConfigureApiAccessMethodViewModelProtocol {
        ConfigureApiAccessMethodViewModel(
            usage: .add,
            accessMethodViewModel: AccessMethodViewModel(),
            interactor: EditAccessMethodInteractor(
                repository: repository, proxyConfigurationTester: proxyConfigurationTester,
                breadcrumbsProvider: breadcrumbsProvider),
        )
    }

    func makeEditMethodViewModel(for item: PersistentAccessMethod) -> any EditAccessMethodViewModelProtocol {
        EditAccessMethodViewModel(
            accessMethod: item,
            interactor: EditAccessMethodInteractor(
                repository: repository,
                proxyConfigurationTester: proxyConfigurationTester,
                breadcrumbsProvider: breadcrumbsProvider))
    }

    func makeConfigureAccessMethodViewModel(for item: PersistentAccessMethod)
        -> any ConfigureApiAccessMethodViewModelProtocol
    {
        ConfigureApiAccessMethodViewModel(
            usage: .edit,
            accessMethodViewModel: AccessMethodViewModel(
                id: item.id, name: item.name, method: item.kind, isEnabled: item.isEnabled,
                socks: item.proxyConfiguration.socksViewModel, shadowsocks: item.proxyConfiguration.shadowsocksViewModel
            ),
            interactor: EditAccessMethodInteractor(
                repository: repository,
                proxyConfigurationTester: proxyConfigurationTester,
                breadcrumbsProvider: breadcrumbsProvider))
    }
}
