//
//  AccessMethodIterator.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings
import MullvadTypes

final class AccessMethodIterator: @unchecked Sendable, SwiftConnectionModeProviding {
    private let dataSource: AccessMethodRepositoryDataSource

    private var index = 0
    private var cancellables = Set<Combine.AnyCancellable>()

    private var enabledConfigurations: [PersistentAccessMethod] {
        dataSource.fetchAll().filter { $0.isEnabled }
    }

    private var lastReachableApiAccessId: UUID? {
        dataSource.fetchLastReachable().id
    }

    public var domainName: String {
        REST.encryptedDNSHostname
    }

    init(dataSource: AccessMethodRepositoryDataSource) {
        self.dataSource = dataSource

        self.dataSource
            .accessMethodsPublisher
            .sink { [weak self] _ in
                guard let self else { return }
                self.refreshCacheIfNeeded()
            }
            .store(in: &cancellables)
    }

    private func refreshCacheIfNeeded() {
        // Validating the index of `lastReachableApiAccessCache` after any changes in `AccessMethodRepository`
        if let firstIndex = enabledConfigurations.firstIndex(where: { $0.id == lastReachableApiAccessId }) {
            index = firstIndex
        }

        dataSource.saveLastReachable(pick())
    }

    // TODO: Only one should decide who rotates, either Swift or Rust. For now, Swift dictates when the methods are rotated
    func rotate() {
        let (partial, isOverflow) = index.addingReportingOverflow(1)
        index = isOverflow ? 0 : partial
        dataSource.saveLastReachable(pick())
    }

    func pick() -> PersistentAccessMethod {
        let configurations = enabledConfigurations
        if configurations.isEmpty {
            /// Returning `Default` strategy  when  all is disabled
            return dataSource.directAccess
        } else {
            /// Picking the next `Enabled` configuration in order they are added
            /// And starting from the beginning when it reaches end
            let circularIndex = index % configurations.count
            return configurations[circularIndex]
        }
    }

    func accessMethods() -> [PersistentAccessMethod] {
        dataSource.fetchAll()
    }
}
