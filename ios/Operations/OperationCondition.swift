//
//  OperationCondition.swift
//  Operations
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol OperationCondition {
    var name: String { get }
    var isMutuallyExclusive: Bool { get }

    func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void)
}
