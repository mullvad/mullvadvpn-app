// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public final class BlockCondition: OperationCondition {
    public typealias HandlerBlock = (Operation, @escaping (Bool) -> Void) -> Void

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

    public func evaluate(for operation: Operation, completion: @escaping (Bool) -> Void) {
        block(operation, completion)
    }
}
