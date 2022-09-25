//
//  OperationSmokeTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 09/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

class OperationSmokeTests: XCTestCase {
    func testBatch() {
        let expect = expectation(description: "Expect all operations to finish.")
        let operationQueue = AsyncOperationQueue()

        let operations = (1 ... 500).flatMap { i -> [Operation] in
            let parent = BlockOperation()
            parent.cancel()

            let child = AsyncBlockOperation {
                print("Execute block operation \(i)")
            }

            child.addDependency(parent)
            child.addCondition(NoFailedDependenciesCondition(ignoreCancellations: true))

            return [parent, child]
        }

        DispatchQueue.global().async {
            operationQueue.addOperations(operations, waitUntilFinished: true)
            expect.fulfill()
        }

        waitForExpectations(timeout: 1)
    }
}
