//
//  Task+AnyCancellable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-08-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

extension Task {
    public var cancellable: AnyCancellable {
        AnyCancellable {
            cancel()
        }
    }
}
