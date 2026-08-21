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

extension Result {
    public var value: Success? {
        switch self {
        case let .success(value):
            return value
        case .failure:
            return nil
        }
    }

    public var error: Failure? {
        switch self {
        case .success:
            return nil
        case let .failure(error):
            return error
        }
    }

    public var isSuccess: Bool {
        switch self {
        case .success:
            return true
        case .failure:
            return false
        }
    }

    public func tryMap<NewSuccess>(_ body: (Success) throws -> NewSuccess) -> Result<NewSuccess, Error> {
        Result<NewSuccess, Error> {
            let value = try self.get()

            return try body(value)
        }
    }

    @discardableResult public func inspectError(_ body: (Failure) -> Void) -> Self {
        if case let .failure(error) = self {
            body(error)
        }
        return self
    }
}

extension Result {
    public func flattenValue<T>() -> T? where Success == T? {
        switch self {
        case let .success(optional):
            return optional.flatMap { $0 }
        case .failure:
            return nil
        }
    }
}
