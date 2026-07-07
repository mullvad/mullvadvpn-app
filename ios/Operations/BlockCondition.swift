//
//  BlockCondition.swift
//  Operations
//
//  Created by pronebird on 25/09/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class BlockCondition: OperationCondition, @unchecked Sendable {

    public typealias HandlerBlock =
        @Sendable (Operation, @escaping @Sendable (Bool) -> Void) -> Void

    public var name: String {
        "BlockCondition"
    }

    public var isMutuallyExclusive: Bool {
        false
    }

    public let block: HandlerBlock

    public init(block: @escaping HandlerBlock) {
        self.block = block
    }

    public func evaluate(
        for operation: Operation,
        completion: @escaping @Sendable (Bool) -> Void
    ) {
        block(operation, completion)
    }
}
