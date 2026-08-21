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
