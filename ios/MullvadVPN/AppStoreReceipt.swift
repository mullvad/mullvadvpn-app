//
//  AppStoreReceipt.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations
import StoreKit

struct AppStoreReceiptNotFound: LocalizedError {
    var errorDescription: String? {
        return "AppStore receipt file does not exist on disk."
    }
}

enum AppStoreReceipt {
    /// Internal operation queue.
    private static let operationQueue: OperationQueue = {
        let queue = AsyncOperationQueue()
        queue.name = "AppStoreReceiptQueue"
        queue.maxConcurrentOperationCount = 1
        return queue
    }()

    /// Read AppStore receipt from disk or refresh it from AppStore if it's missing.
    /// This call may trigger a sign in with AppStore prompt to appear.
    static func fetch(
        forceRefresh: Bool = false,
        receiptProperties: [String: Any]? = nil,
        completionHandler: @escaping (OperationCompletion<Data, Error>) -> Void
    ) -> Cancellable {
        let operation = FetchAppStoreReceiptOperation(
            forceRefresh: forceRefresh,
            receiptProperties: receiptProperties,
            completionHandler: completionHandler
        )

        operation.addObserver(
            BackgroundObserver(name: "Fetch AppStore receipt", cancelUponExpiration: true)
        )

        operationQueue.addOperation(operation)

        return operation
    }
}

private class FetchAppStoreReceiptOperation: ResultOperation<Data, Error>, SKRequestDelegate {
    private var request: SKReceiptRefreshRequest?
    private let receiptProperties: [String: Any]?
    private let forceRefresh: Bool

    init(
        forceRefresh: Bool,
        receiptProperties: [String: Any]?,
        completionHandler: @escaping (Completion) -> Void
    ) {
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties

        super.init(
            dispatchQueue: .main,
            completionQueue: .main,
            completionHandler: completionHandler
        )
    }

    override func main() {
        // Pull receipt from AppStore if requested.
        guard !forceRefresh else {
            startRefreshRequest()
            return
        }

        // Read AppStore receipt from disk.
        do {
            let data = try readReceiptFromDisk()

            finish(completion: .success(data))
        } catch is AppStoreReceiptNotFound {
            // Pull receipt from AppStore if it's not cached locally.
            startRefreshRequest()
        } catch {
            finish(completion: .failure(error))
        }
    }

    override func operationDidCancel() {
        request?.cancel()
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

        if let error = error {
            finish(completion: .failure(error))
        } else {
            let result = Result { try readReceiptFromDisk() }

            finish(completion: OperationCompletion(result: result))
        }
    }

    private func readReceiptFromDisk() throws -> Data {
        guard let appStoreReceiptURL = Bundle.main.appStoreReceiptURL else {
            throw AppStoreReceiptNotFound()
        }

        do {
            return try Data(contentsOf: appStoreReceiptURL)
        } catch let error as CocoaError
            where error.code == .fileReadNoSuchFile || error.code == .fileNoSuchFile
        {
            throw AppStoreReceiptNotFound()
        } catch {
            throw error
        }
    }
}
