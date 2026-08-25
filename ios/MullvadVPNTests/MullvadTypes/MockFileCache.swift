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
actor MockFileCache<Cache: Codable & Equatable & Sendable>: FileCacheProtocol {
    private var state: State

    init(initialState: State = .fileNotFound) {
        state = initialState
    }

    // MARK: - Asynchronous functions

    /// Returns internal state.
    func getState() -> State {
        state
    }

    func read() async throws -> Cache {
        switch state {
        case .fileNotFound:
            throw CocoaError(.fileReadNoSuchFile)
        case let .exists(content):
            return content
        }
    }

    func write(_ content: Cache) async throws {
        state = .exists(content)
    }

    func clear() async throws {
        state = .fileNotFound
    }

    // MARK: - Synchronous shims
    // Will be removed once all call sites have been migrated.

    nonisolated func read() throws -> Cache {
        try SynchRunner.run {
            try await self.read()
        }
    }

    nonisolated func write(_ cache: Cache) throws {
        try SynchRunner.run {
            try await self.write(cache)
        }
    }

    nonisolated func clear() throws {
        try SynchRunner.run {
            try await self.clear()
        }
    }

    // MARK: - Private

    enum State: Equatable {
        /// File does not exist yet.
        case fileNotFound

        /// File exists with the given contents.
        case exists(Cache)

        var isExists: Bool {
            if case .exists = self {
                return true
            } else {
                return false
            }
        }
    }
}
