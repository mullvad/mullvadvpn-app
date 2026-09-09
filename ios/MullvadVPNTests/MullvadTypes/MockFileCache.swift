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
actor MockFileCache<Content: Codable & Equatable & Sendable>: FileCacheProtocol, DispatchSerialQueueActor {
    nonisolated let queue = DispatchSerialQueue(label: "net.mullvad.mockfilecache")

    nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private var state: State

    init(initialState: State = .fileNotFound) {
        state = initialState
    }

    /// Returns internal state.
    func getState() -> State {
        state
    }

    // MARK: - Asynchronous functions

    func read() async throws -> Content {
        try readState()
    }

    func write(_ content: Content) async throws {
        writeState(content)
    }

    func clear() async throws {
        clearState()
    }

    // MARK: - Synchronous functions
    // Only for callers that have not been migrated to async yet.

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    nonisolated func read() throws -> Content {
        try runBlocking { try $0.readState() }
    }

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    nonisolated func write(_ content: Content) throws {
        runBlocking { $0.writeState(content) }
    }

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    nonisolated func clear() throws {
        runBlocking { $0.clearState() }
    }

    // MARK: - Private

    private func readState() throws -> Content {
        switch state {
        case .fileNotFound:
            throw CocoaError(.fileReadNoSuchFile)
        case let .exists(content):
            return content
        }
    }

    private func writeState(_ content: Content) {
        state = .exists(content)
    }

    private func clearState() {
        state = .fileNotFound
    }

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
