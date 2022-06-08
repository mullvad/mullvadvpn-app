//
//  GroupOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class GroupOperation: AsyncOperation {
    private let operationQueue = AsyncOperationQueue()
    private let children: [Operation]

    init(operations: [Operation]) {
        children = operations

        super.init(dispatchQueue: nil)
    }

    override func main() {
        let finishingOperation = BlockOperation()
        finishingOperation.completionBlock = { [weak self] in
            self?.finish()
        }
        finishingOperation.addDependencies(children)

        operationQueue.addOperations(children, waitUntilFinished: false)
        operationQueue.addOperation(finishingOperation)
    }

    override func operationDidCancel() {
        operationQueue.cancelAllOperations()
    }
}
