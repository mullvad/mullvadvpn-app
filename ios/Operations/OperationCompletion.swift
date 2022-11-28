//
//  OperationCompletion.swift
//  Operations
//
//  Created by pronebird on 24/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum OperationCompletion<Success, Failure: Error> {
    case cancelled
    case success(Success)
    case failure(Failure)

    public var isSuccess: Bool {
        if case .success = self {
            return true
        } else {
            return false
        }
    }

    public var value: Success? {
        if case let .success(value) = self {
            return value
        } else {
            return nil
        }
    }

    public var error: Failure? {
        if case let .failure(error) = self {
            return error
        } else {
            return nil
        }
    }

    public var result: Result<Success, Failure>? {
        switch self {
        case let .success(value):
            return .success(value)
        case let .failure(error):
            return .failure(error)
        case .cancelled:
            return nil
        }
    }

    public init(result: Result<Success, Failure>) {
        switch result {
        case let .success(value):
            self = .success(value)
        case let .failure(error):
            self = .failure(error)
        }
    }

    public init(error: Failure?) where Success == Void {
        if let error = error {
            self = .failure(error)
        } else {
            self = .success(())
        }
    }

    public func get() throws -> Success {
        if let result = result {
            return try result.get()
        } else {
            throw OperationCancellationError()
        }
    }

    public func map<NewSuccess>(_ block: (Success) -> NewSuccess)
        -> OperationCompletion<NewSuccess, Failure>
    {
        switch self {
        case let .success(value):
            return .success(block(value))
        case let .failure(error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    public func mapError<NewFailure: Error>(_ block: (Failure) -> NewFailure)
        -> OperationCompletion<Success, NewFailure>
    {
        switch self {
        case let .success(value):
            return .success(value)
        case let .failure(error):
            return .failure(block(error))
        case .cancelled:
            return .cancelled
        }
    }

    public func flatMap<NewSuccess>(_ block: (Success) -> OperationCompletion<NewSuccess, Failure>)
        -> OperationCompletion<NewSuccess, Failure>
    {
        switch self {
        case let .success(value):
            return block(value)
        case let .failure(error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    public func flatMapError<NewFailure: Error>(
        _ block: (Failure)
            -> OperationCompletion<Success, NewFailure>
    ) -> OperationCompletion<Success, NewFailure> {
        switch self {
        case let .success(value):
            return .success(value)
        case let .failure(error):
            return block(error)
        case .cancelled:
            return .cancelled
        }
    }

    public func tryMap<NewSuccess>(_ block: (Success) throws -> NewSuccess)
        -> OperationCompletion<NewSuccess, Error>
    {
        switch self {
        case let .success(value):
            do {
                return .success(try block(value))
            } catch {
                return .failure(error)
            }
        case let .failure(error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    public func ignoreOutput() -> OperationCompletion<Void, Failure> {
        return map { _ in () }
    }

    public func eraseFailureType() -> OperationCompletion<Success, Error> {
        return mapError { $0 }
    }
}

public struct OperationCancellationError: LocalizedError {
    public var errorDescription: String? {
        return "Operation was cancelled."
    }
}
