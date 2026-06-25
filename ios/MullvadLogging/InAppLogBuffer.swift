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
        lock.withLock {
            entries.append(entry)
        }
    }

    public func flush() -> [InAppLogEntry] {
        lock.withLock {
            let result = entries
            entries.removeAll()

            return result
        }
    }
}
