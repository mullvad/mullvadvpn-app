//
//  MullvadAccessMethodReceiver.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-03-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadTypes

public class MullvadAccessMethodReceiver {
    private var cancellables = Set<Combine.AnyCancellable>()
    let apiContext: MullvadApiContext

    public init(
        apiContext: MullvadApiContext,
        accessMethodsDataSource: AnyPublisher<[PersistentAccessMethod], Never>,
        lastReachableDataSource: AnyPublisher<PersistentAccessMethod, Never>
    ) {
        self.apiContext = apiContext

        lastReachableDataSource.sink { [weak self] in
            self?.saveLastReachable($0)
        }
        .store(in: &cancellables)

        accessMethodsDataSource.sink { [weak self] in
            self?.updateAccessMethods($0)
        }.store(in: &cancellables)
    }

    private func saveLastReachable(_ lastReachable: PersistentAccessMethod) {
        mullvad_api_use_access_method(apiContext.context, lastReachable.id.uuidString)
    }

    private func updateAccessMethods(_ accessMethods: [PersistentAccessMethod]) {
        let settingsWrapper = initAccessMethodSettingsWrapper(methods: accessMethods)
        mullvad_api_update_access_methods(apiContext.context, settingsWrapper)
    }
}
