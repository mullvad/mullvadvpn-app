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
    private var state: State

    init(initialState: State = .fileNotFound) {
        state = initialState
    }

    // MARK: - Asynchronous functions

    /// Returns internal state.
    func getState() -> State {
        state
    }

    func read() async throws -> Content {
        switch state {
        case .fileNotFound:
            throw CocoaError(.fileReadNoSuchFile)
        case let .exists(content):
            return content
        }
    }

    func write(_ content: Content) async throws {
        state = .exists(content)
    }

    func clear() async throws {
        state = .fileNotFound
    }

    // MARK: - Synchronous shims
    // Will be removed once all call sites have been migrated.

    nonisolated func read() throws -> Content {
        try FileCache<Content>.SynchRunner.run {
            try await self.read()
        }
    }

    nonisolated func write(_ content: Content) throws {
        try FileCache<Content>.SynchRunner.run {
            try await self.write(content)
        }
    }

    nonisolated func clear() throws {
        try FileCache<Content>.SynchRunner.run {
            try await self.clear()
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
