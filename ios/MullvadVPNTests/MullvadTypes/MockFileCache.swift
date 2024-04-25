//
//  MockFileCache.swift
//  MullvadVPNTests
//
//  Created by pronebird on 13/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
        stateLock.lock()
        defer { stateLock.unlock() }

        return state
    }

    func read() throws -> Content {
        stateLock.lock()
        defer { stateLock.unlock() }

        switch state {
        case .fileNotFound:
            throw CocoaError(.fileReadNoSuchFile)
        case let .exists(content):
            return content
        }
    }

    func write(_ content: Content) throws {
        stateLock.lock()
        defer { stateLock.unlock() }

        state = .exists(content)
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
