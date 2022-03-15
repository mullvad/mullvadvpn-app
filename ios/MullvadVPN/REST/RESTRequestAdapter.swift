//
//  RESTRequestAdapter.swift
//  MullvadVPN
//
//  Created by pronebird on 03/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {

    struct RequestAdapter<Success> {
        typealias CompletionHandler = (Result<Success, REST.Error>) -> Void

        private let block: (RetryStrategy, @escaping CompletionHandler) -> AnyCancellable

        init(block: @escaping (RetryStrategy, @escaping CompletionHandler) -> AnyCancellable) {
            self.block = block
        }

        func execute(retryStrategy: RetryStrategy = RetryStrategy.noRetry, completionHandler: @escaping CompletionHandler) -> AnyCancellable {
            return self.block(retryStrategy, completionHandler)
        }
    }



}
