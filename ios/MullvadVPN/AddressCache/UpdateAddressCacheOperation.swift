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
        private let queue: DispatchQueue
        private let restClient: REST.Client
        private let store: AddressCache.Store
        private let updateInterval: TimeInterval

        private var requestTask: Cancellable?

        init(queue: DispatchQueue, restClient: REST.Client, store: AddressCache.Store, updateInterval: TimeInterval, completionHandler: CompletionHandler?) {
            self.queue = queue
            self.restClient = restClient
            self.store = store
            self.updateInterval = updateInterval

            super.init(completionQueue: queue, completionHandler: completionHandler)
        }

        override func main() {
            queue.async {
                self.startUpdate()
            }
        }

        override func cancel() {
            super.cancel()

            queue.async {
                self.requestTask?.cancel()
                self.requestTask = nil
            }
        }

        private func startUpdate() {
            guard !isCancelled else {
                finish(completion: .cancelled)
                return
            }

            let lastUpdate = store.getLastUpdateDateAndWait()
            let nextUpdate = Date(timeInterval: updateInterval, since: lastUpdate)

            guard nextUpdate <= Date() else {
                finish(completion: .success(.throttled(lastUpdate)))
                return
            }

            requestTask = restClient.getAddressList(retryStrategy: .default) { result in
                self.queue.async {
                    self.handleResponse(result)
                }
            }
        }

        private func handleResponse(_ result: Result<[AnyIPEndpoint], REST.Error>) {
            switch result {
            case .success(let newEndpoints):
                self.store.setEndpoints(newEndpoints) { error in
                    if let error = error {
                        self.finish(completion: .failure(error))
                    } else {
                        self.finish(completion: .success(.finished))
                    }
                }

            case .failure(let error):
                if case URLError.cancelled = error {
                    self.finish(completion: .cancelled)
                } else {
                    self.finish(completion: .failure(error))
                }
            }
        }
    }
}

extension OperationCompletion where Success == AddressCache.CacheUpdateResult {
    var isTaskCompleted: Bool {
        switch self {
        case .success:
            return true
        case .cancelled, .failure:
            return false
        }
    }
}
