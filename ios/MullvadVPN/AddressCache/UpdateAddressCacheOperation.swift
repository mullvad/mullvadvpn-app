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
        /// Operation was cancelled.
        case cancelled

        /// Address cache update was throttled as it was requested too early.
        case throttled(_ lastUpdateDate: Date)

        /// Failure to update address cache.
        case failure(Error)

        /// Address cache is successfully updated.
        case success

        var isTaskCompleted: Bool {
            switch self {
            case .cancelled, .failure:
                return false
            case .success, .throttled:
                return true
            }
        }
    }

    class UpdateAddressCacheOperation: AsyncOperation {
        typealias CompletionHandler = (_ result: CacheUpdateResult) -> Void

        private let queue: DispatchQueue
        private let restClient: REST.Client
        private let store: AddressCache.Store
        private let updateInterval: TimeInterval

        private var completionHandler: CompletionHandler?
        private var restCancellationHandle: Cancellable?

        init(queue: DispatchQueue, restClient: REST.Client, store: AddressCache.Store, updateInterval: TimeInterval, completionHandler: CompletionHandler?) {
            self.queue = queue
            self.restClient = restClient
            self.store = store
            self.updateInterval = updateInterval
            self.completionHandler = completionHandler
        }

        override func cancel() {
            queue.async {
                super.cancel()
                self.restCancellationHandle?.cancel()
            }
        }

        override func main() {
            queue.async {
                self.startUpdate()
            }
        }

        private func startUpdate() {
            guard !isCancelled else {
                completeOperation(with: .cancelled)
                return
            }

            let lastUpdate = store.getLastUpdateDateAndWait()
            let nextUpdate = Date(timeInterval: updateInterval, since: lastUpdate)

            guard nextUpdate <= Date() else {
                completeOperation(with: .throttled(lastUpdate))
                return
            }

            restCancellationHandle = restClient.getAddressList(retryStrategy: .default) { restResult in
                    self.queue.async {
                        switch restResult {
                        case .success(let newEndpoints):
                            self.store.setEndpoints(newEndpoints) { error in
                                self.queue.async {
                                    if let error = error {
                                        self.completeOperation(with: .failure(error))
                                    } else {
                                        self.completeOperation(with: .success)
                                    }
                                }
                            }

                        case .failure(let error):
                            if case URLError.cancelled = error {
                                self.completeOperation(with: .cancelled)
                            } else {
                                self.completeOperation(with: .failure(error))
                            }
                        }
                    }
                }
        }

        private func completeOperation(with result: CacheUpdateResult) {
            completionHandler?(result)
            completionHandler = nil

            finish()
        }
    }
}
