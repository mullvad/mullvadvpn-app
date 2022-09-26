//
//  BlockCondition.swift
//  Operations
//
//  Created by pronebird on 25/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class BlockCondition: OperationCondition {
    public typealias HandlerBlock = (Operation, @escaping (Bool) -> Void) -> Void

    public var name: String {
        return "BlockCondition"
    }

    public var isMutuallyExclusive: Bool {
        return false
    }

    public let block: HandlerBlock
    public init(block: @escaping HandlerBlock) {
        self.block = block
    }

    public func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        block(operation, completion)
    }
}
