//
//  RESTAuthorization.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Operations

protocol RESTAuthorizationProvider {
    typealias Completion = OperationCompletion<REST.Authorization, REST.Error>

    func getAuthorization(completion: @escaping (Completion) -> Void) -> Cancellable
}

extension REST {
    typealias Authorization = String

    struct AccessTokenProvider: RESTAuthorizationProvider {
        private let accessTokenManager: AccessTokenManager
        private let accountNumber: String
        private let retryStrategy: REST.RetryStrategy

        init(
            accessTokenManager: AccessTokenManager,
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) {
            self.accessTokenManager = accessTokenManager
            self.accountNumber = accountNumber
            self.retryStrategy = retryStrategy
        }

        func getAuthorization(completion: @escaping (Completion) -> Void) -> Cancellable {
            return accessTokenManager.getAccessToken(
                accountNumber: accountNumber,
                retryStrategy: retryStrategy
            ) { operationCompletion in
                completion(operationCompletion.map { tokenData in
                    return tokenData.accessToken
                })
            }
        }
    }
}

extension REST.Proxy where ConfigurationType == REST.AuthProxyConfiguration {
    func createAuthorizationProvider(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) -> RESTAuthorizationProvider {
        return REST.AccessTokenProvider(
            accessTokenManager: configuration.accessTokenManager,
            accountNumber: accountNumber,
            retryStrategy: retryStrategy
        )
    }
}
