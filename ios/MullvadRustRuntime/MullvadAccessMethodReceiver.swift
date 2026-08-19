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
import Foundation
import MullvadTypes

public class MullvadAccessMethodReceiver {
    private var cancellables = Set<Combine.AnyCancellable>()
    let apiContext: MullvadApiContext
    let validShadowsocksCiphers: [String]

    public init(
        apiContext: MullvadApiContext,
        validShadowsocksCiphers: [String],
        accessMethodsDataSource: AnyPublisher<[PersistentAccessMethod], Never>,
        requestDataSource: AnyPublisher<PersistentAccessMethod, Never>
    ) {
        self.apiContext = apiContext
        self.validShadowsocksCiphers = validShadowsocksCiphers

        requestDataSource.sink { [weak self] latestReachable in
            self?.saveLastReachable(latestReachable)
        }
        .store(in: &cancellables)

        accessMethodsDataSource.sink { [weak self] in
            self?.updateAccessMethods($0)
        }.store(in: &cancellables)
    }

    private func saveLastReachable(_ lastReachable: PersistentAccessMethod) {
        mullvadApiUseAccessMethod(
            apiContext: apiContext.context,
            id: lastReachable.id.uuidString)
    }

    private func updateAccessMethods(_ accessMethods: [PersistentAccessMethod]) {
        let settingsWrapper = initAccessMethodSettingsWrapper(methods: accessMethods)
        mullvadApiUpdateAccessMethods(
            apiContext: apiContext.context,
            settingsWrapper: settingsWrapper)
    }
}
