//
//  AppStoreReceipt.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum AppStoreReceipt {
    enum Error: ChainedError {
        /// AppStore receipt file does not exist or file URL is not available.
        case doesNotExist

        /// IO error.
        case io(Swift.Error)

        /// Failure to refresh the receipt from AppStore.
        case refresh(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .doesNotExist:
                return "AppStore receipt file does not exist on disk."
            case .io:
                return "Read error."
            case .refresh:
                return "Receipt refresh error."
            }
        }
    }

    /// Internal dispatch queue.
    private static let dispatchQueue = DispatchQueue(label: "AppStoreReceiptDispatchQueue")

    /// Internal operation queue.
    private static let operationQueue: OperationQueue = {
        let queue = OperationQueue()
        queue.name = "AppStoreReceiptQueue"
        queue.maxConcurrentOperationCount = 1
        return queue
    }()

    /// Read AppStore receipt from disk or refresh it from AppStore if it's missing.
    /// This call may trigger a sign in with AppStore prompt to appear.
    static func fetch(forceRefresh: Bool = false, receiptProperties: [String: Any]? = nil, completionHandler: @escaping (OperationCompletion<Data, Error>) -> Void) -> AnyCancellable {
        let operation = FetchAppStoreReceiptOperation(
            dispatchQueue: dispatchQueue,
            forceRefresh: forceRefresh,
            receiptProperties: receiptProperties,
            completionHandler: completionHandler
        )

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Fetch AppStore receipt") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        operationQueue.addOperation(operation)

        return AnyCancellable {
            operation.cancel()
        }
    }
}

fileprivate class FetchAppStoreReceiptOperation: AsyncOperation, SKRequestDelegate {
    private var request: SKReceiptRefreshRequest?
    private let receiptProperties: [String: Any]?
    private let forceRefresh: Bool

    private let dispatchQueue: DispatchQueue
    private var completionHandler: ((OperationCompletion<Data, AppStoreReceipt.Error>) -> Void)?

    init(dispatchQueue: DispatchQueue, forceRefresh: Bool, receiptProperties: [String: Any]?, completionHandler: @escaping (OperationCompletion<Data, AppStoreReceipt.Error>) -> Void) {
        self.dispatchQueue = dispatchQueue
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties
        self.completionHandler = completionHandler
    }

    override func main() {
        dispatchQueue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            // Pull receipt from AppStore if requested.
            guard !self.forceRefresh else {
                self.startRefreshRequest()
                return
            }

            // Read AppStore receipt from disk.
            let result = self.readReceiptFromDisk()

            // Pull receipt from AppStore if it's not cached locally.
            if case .failure(.doesNotExist) = result {
                self.startRefreshRequest()
            } else {
                self.finish(completion: OperationCompletion(result: result))
            }
        }
    }

    override func cancel() {
        dispatchQueue.async {
            super.cancel()

            self.request?.cancel()
        }
    }

    // - MARK: SKRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        dispatchQueue.async {
            self.didFinishRefreshRequest(error: nil)
        }
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        dispatchQueue.async {
            self.didFinishRefreshRequest(error: error)
        }
    }

    // - MARK: Private

    private func startRefreshRequest() {
        let request = SKReceiptRefreshRequest(receiptProperties: receiptProperties)
        request.delegate = self
        request.start()

        self.request = request
    }

    private func didFinishRefreshRequest(error: Error?) {
        guard !isCancelled else {
            finish(completion: .cancelled)
            return
        }

        let result: Result<Data, AppStoreReceipt.Error>

        if let error = error {
            result = .failure(.refresh(error))
        } else {
            result = readReceiptFromDisk()
        }

        finish(completion: OperationCompletion(result: result))
    }

    private func readReceiptFromDisk() -> Result<Data, AppStoreReceipt.Error> {
        guard let appStoreReceiptURL = Bundle.main.appStoreReceiptURL else {
            return .failure(.doesNotExist)
        }

        let readResult = Result { try Data(contentsOf: appStoreReceiptURL) }

        return readResult.mapError { (error) -> AppStoreReceipt.Error in
            if let cocoaError = error as? CocoaError, cocoaError.code == .fileReadNoSuchFile || cocoaError.code == .fileNoSuchFile {
                return .doesNotExist
            } else {
                return .io(error)
            }
        }
    }

    private func finish(completion: OperationCompletion<Data, AppStoreReceipt.Error>) {
        let block = completionHandler
        completionHandler = nil

        DispatchQueue.main.async {
            block?(completion)
        }

        finish()
    }

}
