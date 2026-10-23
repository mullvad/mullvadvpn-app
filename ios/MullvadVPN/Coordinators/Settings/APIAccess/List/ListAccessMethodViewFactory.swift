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

    init(
        repository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol
    ) {
        self.repository = repository
        self.proxyConfigurationTester = proxyConfigurationTester
    }

    func makeListViewModel() -> any ListAccessViewModelProtocol {
        ListAccessViewModel(
            interactor: ListAccessMethodInteractor(
                repository: repository
            )
        )
    }

    func makeAddMethodViewModel() -> any ConfigureApiAccessMethodViewModelProtocol {
        ConfigureApiAccessMethodViewModel(accessMethod: nil)
    }

    func makeEditMethodViewModel(for item: PersistentAccessMethod) -> any EditAccessMethodViewModelProtocol {
        EditAccessMethodViewModel(
            accessMethod: item,
            interactor: EditAccessMethodInteractor(
                repository: repository
            ),
            proxyConfigurationTester: proxyConfigurationTester
        )
    }

    func makeConfigureAccessMethodViewModel(for item: PersistentAccessMethod)
        -> any ConfigureApiAccessMethodViewModelProtocol
    {
        ConfigureApiAccessMethodViewModel(accessMethod: item)
    }
}
