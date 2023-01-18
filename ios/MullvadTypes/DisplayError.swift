//
//  DisplayError.swift
//  MullvadTypes
//
//  Created by pronebird on 17/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A protocol that adds a formal interface for all errors displayed in user interface.
///
/// This protocol is meant to be used in place of `LocalizedError` when producing a user friendly
/// error message that requires a deeper look at the underlying cause.
///
/// Note that `Logger.error(error: Error)` picks up `errorDescription`s when unrolling
/// the underlying error chain, hence it's better to keep error descriptions relatively concise,
/// explaining what happened but without telling why that happened.
public protocol DisplayError {
    var displayErrorDescription: String? { get }
}
