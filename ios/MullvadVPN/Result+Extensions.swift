//
//  Result+Extensions.swift
//  MullvadVPN
//
//  Created by pronebird on 15/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Result {
    var value: Success? {
        switch self {
        case let .success(value):
            return value
        case .failure:
            return nil
        }
    }

    var error: Failure? {
        switch self {
        case .success:
            return nil
        case let .failure(error):
            return error
        }
    }

    var isSuccess: Bool {
        switch self {
        case .success:
            return true
        case .failure:
            return false
        }
    }

    func tryMap<NewSuccess>(_ body: (Success) throws -> NewSuccess) -> Result<NewSuccess, Error> {
        return Result<NewSuccess, Error> {
            let value = try self.get()

            return try body(value)
        }
    }

    @discardableResult func inspectError(_ body: (Failure) -> Void) -> Self {
        if case let .failure(error) = self {
            body(error)
        }
        return self
    }
}

extension Result {
    func flattenValue<T>() -> T? where Success == T? {
        switch self {
        case let .success(optional):
            return optional.flatMap { $0 }
        case .failure:
            return nil
        }
    }
}
