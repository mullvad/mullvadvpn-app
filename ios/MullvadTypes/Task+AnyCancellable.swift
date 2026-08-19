//
//  Task+AnyCancellable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-08-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

// Bridge to allow new async/await pattern via Task to be compatible with
// old task grouping where Cancellable is expected.
extension Task {
    public var cancellable: AnyCancellable {
        AnyCancellable {
            cancel()
        }
    }
}
