//
//  UpdateAddressCacheOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension AddressCache {

    enum CacheUpdateResult {
        /// Address cache update was throttled as it was requested too early.
        case throttled(_ lastUpdateDate: Date)

        /// Address cache is successfully updated.
        case finished
    }

    class UpdateAddressCacheOperation: ResultOperation<CacheUpdateResult, Error> {
        private let apiProxy: REST.APIProxy
        private let store: AddressCache.Store
        private let updateInterval: TimeInterval

        private var requestTask: Cancellable?

        init(
            dispatchQueue: DispatchQueue,
            apiProxy: REST.APIProxy,
            store: AddressCache.Store,
            updateInterval: TimeInterval,
            completionHandler: CompletionHandler?
        )
        {
            self.apiProxy = apiProxy
            self.store = store
            self.updateInterval = updateInterval

            super.init(
                dispatchQueue: dispatchQueue,
                completionQueue: dispatchQueue,
                completionHandler: completionHandler
            )
        }

        override func main() {
            let lastUpdate = store.getLastUpdateDate()
            let nextUpdate = Date(timeInterval: updateInterval, since: lastUpdate)

            guard nextUpdate <= Date() else {
                finish(completion: .success(.throttled(lastUpdate)))
                return
            }

            requestTask = apiProxy.getAddressList(retryStrategy: .default) { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.handleResponse(completion)
                }
            }
        }

        override func operationDidCancel() {
            requestTask?.cancel()
            requestTask = nil
        }

        private func handleResponse(_ completion: OperationCompletion<[AnyIPEndpoint], REST.Error>) {
            let mappedCompletion = completion
                .flatMapError { error -> OperationCompletion<[AnyIPEndpoint], Error> in
                    if case URLError.cancelled = error {
                        return .cancelled
                    } else {
                        return .failure(error)
                    }
                }
                .tryMap { endpoints -> CacheUpdateResult in
                    try store.setEndpoints(endpoints)

                    return .finished
                }

            finish(completion: mappedCompletion)
        }
    }
}
