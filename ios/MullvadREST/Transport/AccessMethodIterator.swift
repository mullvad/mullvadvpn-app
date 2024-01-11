//
//  AccessMethodIterator.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings

class AccessMethodIterator {
    private var lastReachableApiAccessCache: LastReachableApiAccessCache
    private let dataSource: AccessMethodRepositoryDataSource

    private var cancellables = Set<Combine.AnyCancellable>()

    /// `UserDefaults` key shared by both processes. Used to cache and synchronize last reachable api access method between them.
    private let lastReachableConfigurationCacheKey = "LastReachableConfigurationCacheKey"

    private var index = 0
    private var enabledConfigurations: [PersistentAccessMethod] = []

    private var lastReachableApiAccessId: UUID {
        lastReachableApiAccessCache.id
    }

    init(_ userDefaults: UserDefaults, dataSource: AccessMethodRepositoryDataSource) {
        self.dataSource = dataSource
        self.lastReachableApiAccessCache = LastReachableApiAccessCache(
            key: lastReachableConfigurationCacheKey,
            defaultValue: dataSource.directAccess.id,
            container: userDefaults
        )

        self.dataSource
            .publisher
            .sink { [weak self] configurations in
                guard let self else { return }
                self.enabledConfigurations = configurations.filter { $0.isEnabled }
                self.refreshCacheIfNeeded()
            }
            .store(in: &cancellables)
    }

    var current: PersistentAccessMethod {
        if enabledConfigurations.isEmpty {
            /// Returning  `Default` strategy  when  all is disabled
            return dataSource.directAccess
        } else {
            /// Picking the next `Enabled` configuration in order they are added
            /// And starting from the beginning when it reaches end
            let circularIndex = index % enabledConfigurations.count
            return enabledConfigurations[circularIndex]
        }
    }

    private func refreshCacheIfNeeded() {
        /// Validating the index of `lastReachableApiAccessCache` after any changes in `AccessMethodRepository`
        if let idx = enabledConfigurations.firstIndex(where: { $0.id == self.lastReachableApiAccessId }) {
            index = idx
        } else {
            /// When `idx` is `nil`, that means the current configuration is not valid any more
            /// Invalidating cache by replacing the `current`  to the next enabled access method
            lastReachableApiAccessCache.id = current.id
        }
    }

    func next() {
        let (partial, isOverflow) = index.addingReportingOverflow(1)
        index = isOverflow ? 0 : partial
        lastReachableApiAccessCache.id = current.id
    }
}
