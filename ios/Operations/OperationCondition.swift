//
//  OperationCondition.swift
//  Operations
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol OperationCondition {
    var name: String { get }
    var isMutuallyExclusive: Bool { get }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void)
}

public extension OperationCondition {
    func evaluate(for operation: Operation) async -> Bool {
        await withCheckedContinuation { continuation in
            evaluate(for: operation) { result in
                continuation.resume(returning: result)
            }
        }
    }
}
