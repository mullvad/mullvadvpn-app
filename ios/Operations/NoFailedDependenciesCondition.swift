//
//  NoFailedDependenciesCondition.swift
//  Operations
//
//  Created by pronebird on 25/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class NoFailedDependenciesCondition: OperationCondition {
    public var name: String {
        return "NoFailedDependenciesCondition"
    }

    public var isMutuallyExclusive: Bool {
        return false
    }

    public let ignoreCancellations: Bool
    public init(ignoreCancellations: Bool) {
        self.ignoreCancellations = ignoreCancellations
    }

    public func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
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
