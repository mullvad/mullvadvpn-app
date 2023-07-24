//
//  AttemptsRecording.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-07-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Defines a generic way to record an attempt.
///
/// Used by `TransportStrategy` to record failed connection attempts in cache.
public protocol AttemptsRecording {
    func record(_ attempts: Int)
}
