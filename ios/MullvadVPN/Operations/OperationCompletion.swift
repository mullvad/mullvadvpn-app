//
//  OperationCompletion.swift
//  MullvadVPN
//
//  Created by pronebird on 24/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum OperationCompletion<Success, Failure: Error> {
    case cancelled
    case success(Success)
    case failure(Failure)

    var isSuccess: Bool {
        if case .success = self {
            return true
        } else {
            return false
        }
    }

    var value: Success? {
        if case .success(let value) = self {
            return value
        } else {
            return nil
        }
    }

    var error: Failure? {
        if case .failure(let error) = self {
            return error
        } else {
            return nil
        }
    }

    var result: Result<Success, Failure>? {
        switch self {
        case .success(let value):
            return .success(value)
        case .failure(let error):
            return .failure(error)
        case .cancelled:
            return nil
        }
    }

    init(result: Result<Success, Failure>) {
        switch result {
        case .success(let value):
            self = .success(value)
        case .failure(let error):
            self = .failure(error)
        }
    }

    func map<NewSuccess>(_ block: (Success) -> NewSuccess) -> OperationCompletion<NewSuccess, Failure> {
        switch self {
        case .success(let value):
            return .success(block(value))
        case .failure(let error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    func mapError<NewFailure: Error>(_ block: (Failure) -> NewFailure) -> OperationCompletion<Success, NewFailure> {
        switch self {
        case .success(let value):
            return .success(value)
        case .failure(let error):
            return .failure(block(error))
        case .cancelled:
            return .cancelled
        }
    }

    func flatMap<NewSuccess>(_ block: (Success) -> OperationCompletion<NewSuccess, Failure>) -> OperationCompletion<NewSuccess, Failure> {
        switch self {
        case .success(let value):
            return block(value)
        case .failure(let error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    func flatMapError<NewFailure: Error>(_ block: (Failure) -> OperationCompletion<Success, NewFailure>) -> OperationCompletion<Success, NewFailure> {
        switch self {
        case .success(let value):
            return .success(value)
        case .failure(let error):
            return block(error)
        case .cancelled:
            return .cancelled
        }
    }

    func tryMap<NewSuccess>(_ block: (Success) throws -> NewSuccess) -> OperationCompletion<NewSuccess, Error> {
        switch self {
        case .success(let value):
            do {
                return .success(try block(value))
            } catch {
                return .failure(error)
            }
        case .failure(let error):
            return .failure(error)
        case .cancelled:
            return .cancelled
        }
    }

    func assertNoSuccess<NewSuccess>() -> OperationCompletion<NewSuccess, Failure> {
        return map { success in
            return success as! NewSuccess
        }
    }

    func assertFailure<NewFailure: Error>(_ failureType: NewFailure.Type)
        -> OperationCompletion<Success, NewFailure>
    {
        return mapError { error -> NewFailure in
            return error as! NewFailure
        }
    }
}
