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

public enum OperationError: LocalizedError, Equatable {
    /// Unsatisfied operation requirement.
    case unsatisfiedRequirement

    /// Operation cancelled.
    case cancelled

    public var errorDescription: String? {
        switch self {
        case .unsatisfiedRequirement:
            return "Unsatisfied operation requirement."
        case .cancelled:
            return "Operation was cancelled."
        }
    }
}

extension Error {
    public var isOperationCancellationError: Bool {
        (self as? OperationError) == .cancelled
    }
}
