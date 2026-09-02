// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

/// File cache actor that simulates file state for use in tests.
actor MockFileCache<Content: Codable & Equatable & Sendable>: FileCacheProtocol {
    /// Only accessed while running on `queue`, mirroring `FileCache`.
    private final class Storage: @unchecked Sendable {
        var state: State

        init(state: State) {
            self.state = state
        }
    }

    nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private nonisolated let storage: Storage
    private nonisolated let queue = DispatchSerialQueue(label: "net.mullvad.MockFileCache")

    init(initialState: State = .fileNotFound) {
        storage = Storage(state: initialState)
    }

    // MARK: - Asynchronous functions

    /// Returns internal state.
    func getState() -> State {
        storage.state
    }

    func read() async throws -> Content {
        try readOnQueue()
    }

    func write(_ content: Content) async throws {
        storage.state = .exists(content)
    }

    func clear() async throws {
        storage.state = .fileNotFound
    }

    // MARK: - Synchronous shims
    // Will be removed once all call sites have been migrated to async/await.

    nonisolated func read() throws -> Content {
        try queue.sync { try readOnQueue() }
    }

    nonisolated func write(_ content: Content) throws {
        queue.sync { storage.state = .exists(content) }
    }

    nonisolated func clear() throws {
        queue.sync { storage.state = .fileNotFound }
    }

    private nonisolated func readOnQueue() throws -> Content {
        switch storage.state {
        case .fileNotFound:
            throw CocoaError(.fileReadNoSuchFile)
        case let .exists(content):
            return content
        }
    }

    // MARK: - Private

    enum State: Equatable {
        /// File does not exist yet.
        case fileNotFound

        /// File exists with the given contents.
        case exists(Content)

        var isExists: Bool {
            if case .exists = self {
                return true
            } else {
                return false
            }
        }
    }
}
