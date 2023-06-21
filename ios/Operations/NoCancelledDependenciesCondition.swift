//
//  NoCancelledDependenciesCondition.swift
//  Operations
//
//  Created by pronebird on 25/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class NoCancelledDependenciesCondition: OperationCondition {
    public var name: String {
        "NoCancelledDependenciesCondition"
    }

    public var isMutuallyExclusive: Bool {
        false
    }

    public init() {}

    public func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        let satisfy = operation.dependencies.allSatisfy { operation in
            !operation.isCancelled
        }

        completion(satisfy)
    }
}
