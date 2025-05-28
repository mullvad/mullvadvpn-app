//
//  ProxyConfigurationTester.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes

/// A concrete implementation of an access method proxy configuration.
class ProxyConfigurationTester: ProxyConfigurationTesterProtocol {
    private var cancellable: MullvadTypes.Cancellable?
    private let transportProvider: ProxyConfigurationTransportProvider
    private var headRequest: REST.APIAvailabilityTestRequest?
    private let apiProxy: APIQuerying

    init(transportProvider: ProxyConfigurationTransportProvider, apiProxy: APIQuerying) {
        self.transportProvider = transportProvider
        self.apiProxy = apiProxy
    }

    func start(configuration: PersistentAccessMethod, completion: @escaping @Sendable (Error?) -> Void) {
        #if DEBUG
        cancellable = apiProxy.checkApiAvailability(retryStrategy: .noRetry, accessMethod: configuration) { success in
            switch success {
            case .success: completion(nil)
            case let .failure(error): completion(error)
            }
        }
        #else
        do {
            let transport = try transportProvider.makeTransport(with: configuration.proxyConfiguration)
            let request = REST.APIAvailabilityTestRequest(transport: transport)
            headRequest = request
            cancellable = request.makeRequest { error in
                DispatchQueue.main.async {
                    completion(error)
                }
            }
        } catch {
            completion(error)
        }
        #endif
    }

    func cancel() {
        cancellable?.cancel()
        cancellable = nil
        headRequest = nil
    }
}
