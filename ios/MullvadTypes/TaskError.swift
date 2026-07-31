//
//  TaskError.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-08-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public enum TaskError: Error {
    case cancelled

    public var errorDescription: String? {
        switch self {
        case .cancelled:
            return "Task was cancelled"
        }
    }
}

extension Error {
    public var isTaskCancellationError: Bool {
        (self as? TaskError) == .cancelled
    }
}
