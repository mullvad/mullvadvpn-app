//
//  MullvadAccessMethodChangeListening.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-07-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

// A protocol that listens for notifications of when the current access method has changed. It receives only the UUID of the new method.
public protocol MullvadAccessMethodChangeListening: AnyObject {
    func accessMethodChangedTo(_ uuid: UUID)
}
