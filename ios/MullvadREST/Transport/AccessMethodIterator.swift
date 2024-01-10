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

    private var lastReachableApiAccessId: UUID {
        lastReachableApiAccessCache.id
    }

    private var index = 0
    private var configurations: [PersistentAccessMethod] = []

    init(_ userDefaults: UserDefaults, dataSource: AccessMethodRepositoryDataSource) {
        self.dataSource = dataSource
        self.lastReachableApiAccessCache = LastReachableApiAccessCache(
            key: lastReachableConfigurationCacheKey,
            defaultValue: dataSource.directAccess.id,
            container: userDefaults
        )

        self.dataSource
            .publisher
            .sink { [weak self] newValue in
                guard let self else { return }
                self.configurations = newValue
                self.refreshCacheIfNeeded()
            }
            .store(in: &cancellables)
    }

    var current: PersistentAccessMethod {
        if enabledConfigurations.isEmpty {
            return dataSource.directAccess
        } else {
            let circularIndex = index % enabledConfigurations.count
            return enabledConfigurations[circularIndex]
        }
    }

    private var enabledConfigurations: [PersistentAccessMethod] {
        return configurations.filter { $0.isEnabled }
    }

    private func refreshCacheIfNeeded() {
        /// updating the cursor whenever the enabled configurations are updated
        guard let idx = self.enabledConfigurations.firstIndex(where: {
            $0.id == self.lastReachableApiAccessId
        }) else {
            self.lastReachableApiAccessCache.id = self.current.id
            return
        }
        self.index = idx
    }

    /// Picking the next `Enabled` configuration in order they are added
    func next() {
        if !enabledConfigurations.isEmpty {
            index += 1
            lastReachableApiAccessCache.id = current.id
        } else {
            index = 0
        }
    }
}
