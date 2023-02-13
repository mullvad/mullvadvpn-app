//
//  OperationError.swift
//  Operations
//
//  Created by pronebird on 24/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

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
        return (self as? OperationError) == .cancelled
    }
}
