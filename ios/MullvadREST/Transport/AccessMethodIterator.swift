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

    private var index = 0
    private var cancellables = Set<Combine.AnyCancellable>()

    private var enabledConfigurations: [PersistentAccessMethod] {
        dataSource.accessMethods.filter { $0.isEnabled }
    }

    private var lastReachableApiAccessId: UUID {
        lastReachableApiAccessCache.id
    }

    init(_ userDefaults: UserDefaults, dataSource: AccessMethodRepositoryDataSource) {
        self.dataSource = dataSource
        self.lastReachableApiAccessCache = LastReachableApiAccessCache(
            defaultValue: dataSource.directAccess.id,
            container: userDefaults
        )

        self.dataSource
            .publisher
            .sink { [weak self] _ in
                guard let self else { return }
                self.refreshCacheIfNeeded()
            }
            .store(in: &cancellables)
    }

    private func refreshCacheIfNeeded() {
        /// Validating the index of `lastReachableApiAccessCache` after any changes in `AccessMethodRepository`
        if let firstIndex = enabledConfigurations.firstIndex(where: { $0.id == self.lastReachableApiAccessId }) {
            index = firstIndex
        } else {
            /// When `firstIndex` is `nil`, that means the current configuration is not valid anymore
            /// Invalidating cache by replacing the `current`  to the next enabled access method
            lastReachableApiAccessCache.id = pick().id
        }
    }

    func rotate() {
        let (partial, isOverflow) = index.addingReportingOverflow(1)
        index = isOverflow ? 0 : partial
        lastReachableApiAccessCache.id = pick().id
    }

    func pick() -> PersistentAccessMethod {
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
}
