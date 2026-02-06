//
//  ProxyConfigurationTester.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes

/// A concrete implementation of an access method proxy configuration.
class ProxyConfigurationTester: ProxyConfigurationTesterProtocol {
    private var cancellable: MullvadTypes.Cancellable?
    private let apiProxy: APIQuerying

    init(apiProxy: APIQuerying) {
        self.apiProxy = apiProxy
    }

    func start(configuration: PersistentAccessMethod, completion: @escaping @Sendable (Error?) -> Void) {
        cancellable = apiProxy.checkApiAvailability(retryStrategy: .noRetry, accessMethod: configuration) { success in
            switch success {
            case .success: completion(nil)
            case let .failure(error): completion(error)
            }
        }
    }

    func cancel() {
        cancellable?.cancel()
        cancellable = nil
    }
}
