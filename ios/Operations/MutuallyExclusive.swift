//
//  MutuallyExclusive.swift
//  Operations
//
//  Created by pronebird on 25/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class MutuallyExclusive: OperationCondition {
    public let name: String

    public var isMutuallyExclusive: Bool {
        return true
    }

    public init(category: String) {
        name = "MutuallyExclusive<\(category)>"
    }

    public func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        completion(true)
    }
}
