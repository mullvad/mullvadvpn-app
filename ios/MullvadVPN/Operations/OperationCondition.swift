//
//  OperationCondition.swift
//  MullvadVPN
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol OperationCondition {
    var name: String { get }
    var isMutuallyExclusive: Bool { get }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void)
}

final class NoCancelledDependenciesCondition: OperationCondition {
    var name: String {
        return "NoCancelledDependenciesCondition"
    }

    var isMutuallyExclusive: Bool {
        return false
    }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        let satisfy = operation.dependencies.allSatisfy { operation in
            return !operation.isCancelled
        }

        completion(satisfy)
    }
}

final class NoFailedDependenciesCondition: OperationCondition {
    var name: String {
        return "NoFailedDependenciesCondition"
    }

    var isMutuallyExclusive: Bool {
        return false
    }

    let ignoreCancellations: Bool
    init(ignoreCancellations: Bool) {
        self.ignoreCancellations = ignoreCancellations
    }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        let satisfy = operation.dependencies.allSatisfy { operation in
            if let operation = operation as? AsyncOperation, operation.error != nil {
                return false
            }

            if operation.isCancelled, !self.ignoreCancellations {
                return false
            }

            return true
        }

        completion(satisfy)
    }
}

final class BlockCondition: OperationCondition {
    typealias HandlerBlock = (Operation, @escaping (Bool) -> Void) -> Void

    var name: String {
        return "BlockCondition"
    }

    var isMutuallyExclusive: Bool {
        return false
    }

    let block: HandlerBlock
    init(block: @escaping HandlerBlock) {
        self.block = block
    }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        block(operation, completion)
    }
}

final class MutuallyExclusive: OperationCondition {
    let name: String

    var isMutuallyExclusive: Bool {
        return true
    }

    init(category: String) {
        name = "MutuallyExclusive<\(category)>"
    }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        completion(true)
    }
}
