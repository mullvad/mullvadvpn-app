//
//  LogRedacting.swift
//  MullvadLogging
//
//  Created by Emīls on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A type that can redact sensitive information from strings.
public protocol LogRedacting: Sendable {
    /// Returns a copy of `string` with sensitive information replaced by placeholders.
    func redact(_ string: String) -> String
}
