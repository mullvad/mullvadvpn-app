//
//  OperationInputInjectionTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 09/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

class OperationInputInjectionTests: XCTestCase {
    func testInject() throws {
        let provider = ResultBlockOperation<Int, Error> {
            return 1
        }

        let consumer = TransformOperation<Int, Int, Error> { input in
            return input + 1
        }

        consumer.inject(from: provider)

        let operationQueue = AsyncOperationQueue()

        operationQueue.addOperations([provider, consumer], waitUntilFinished: true)

        XCTAssertEqual(consumer.output, 2)
    }

    func testInjectVia() throws {
        let provider = ResultBlockOperation<Int, Error> {
            return 1
        }

        let consumer = TransformOperation<String, Int, Error> { input in
            return Int(input)!
        }

        consumer.inject(from: provider) { output in
            return "\(output)"
        }

        let operationQueue = AsyncOperationQueue()

        operationQueue.addOperations([provider, consumer], waitUntilFinished: true)

        XCTAssertEqual(consumer.output, 1)
    }

    func testInjectMany() throws {
        struct Context: OperationInputContext {
            var a: Int?
            var b: Int?

            func reduce() -> Int? {
                guard let a = a, let b = b else { return nil }

                return a + b
            }
        }

        let operationQueue = AsyncOperationQueue()

        let providerA = ResultBlockOperation<Int, Error> {
            return 1
        }

        let providerB = ResultBlockOperation<Int, Error> {
            return 2
        }

        let consumer = TransformOperation<Int, String, Error> { input in
            return "\(input)"
        }

        consumer.injectMany(context: Context())
            .inject(from: providerA, assignOutputTo: \.a)
            .inject(from: providerB, assignOutputTo: \.b)
            .reduce()

        operationQueue.addOperations(
            [providerA, providerB, consumer],
            waitUntilFinished: true
        )

        XCTAssertEqual(consumer.output, "3")
    }
}
