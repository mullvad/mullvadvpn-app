//
//  Gate.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

actor Gate {
    private var isOpen = false
    private var continuation: CheckedContinuation<Void, Never>?

    func wait() async {
        if isOpen {
            return
        }

        await withCheckedContinuation { continuation in
            self.continuation = continuation
        }
    }

    func open() {
        isOpen = true
        continuation?.resume()
        continuation = nil
    }
}
