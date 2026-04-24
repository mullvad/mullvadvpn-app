//
//  InAppLogBuffer.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class InAppLogBuffer: @unchecked Sendable {
    private var entries: [InAppLogEntry] = []
    private let lock = NSLock()

    public init() {}

    public func append(_ entry: InAppLogEntry) {
        lock.lock()
        entries.append(entry)
        lock.unlock()
    }

    public func drain() -> [InAppLogEntry] {
        lock.lock()
        let result = entries
        entries.removeAll()
        lock.unlock()
        return result
    }
}
