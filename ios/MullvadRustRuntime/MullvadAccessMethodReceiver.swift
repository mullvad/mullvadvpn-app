//
//  MullvadAccessMethodReceiver.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-03-31.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
        mullvad_api_use_access_method(apiContext.context, lastReachable.id.uuidString)
    }

    private func updateAccessMethods(_ accessMethods: [PersistentAccessMethod]) {
        let settingsWrapper = initAccessMethodSettingsWrapper(
            methods: accessMethods,
            validShadowsocksCiphers: validShadowsocksCiphers
        )
        mullvad_api_update_access_methods(apiContext.context, settingsWrapper)
    }
}
