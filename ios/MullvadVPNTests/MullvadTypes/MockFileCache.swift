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

/// File cache implementation that simulates file state and uses internal lock to synchronize access to it.
final class MockFileCache<Content: Codable & Equatable>: FileCacheProtocol {
    private var state: State
    private let stateLock = NSLock()

    init(initialState: State = .fileNotFound) {
        state = initialState
    }

    /// Returns internal state.
    func getState() -> State {
        stateLock.withLock {
            state
        }
    }

    func read() throws -> Content {
        try stateLock.withLock {
            switch state {
            case .fileNotFound:
                throw CocoaError(.fileReadNoSuchFile)
            case let .exists(content):
                return content
            }
        }
    }

    func write(_ content: Content) throws {
        stateLock.withLock {
            state = .exists(content)
        }
    }

    func clear() throws {
        stateLock.withLock {
            state = .fileNotFound
        }
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
