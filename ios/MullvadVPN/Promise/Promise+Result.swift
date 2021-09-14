//
//  Promise+Result.swift
//  Promise+Result
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

typealias _Promise = Promise

extension Result {
    typealias Promise = _Promise<Result<Success, Failure>>
}

extension Promise where Value: AnyResult {
    typealias Success = Value.Success
    typealias Failure = Value.Failure

    static func failure(_ error: Failure) -> Result<Success, Failure>.Promise {
        return Result<Success, Failure>.Promise(value: .failure(error))
    }

    static func success(_ value: Success) -> Result<Success, Failure>.Promise {
        return Result<Success, Failure>.Promise(value: .success(value))
    }

    /// Replace value in Result. Passes failure result downstream.
    func setOutput<NewSuccess>(_ newValue: NewSuccess) -> Result<NewSuccess, Failure>.Promise {
        return map { _ in
            return newValue
        }
    }
    /// Returns a Promise containing resolved value or nil.
    func success() -> Promise<Success?> {
        return then { result -> Success? in
            switch result.asConcreteType() {
            case .success(let value):
                return value
            case .failure:
                return nil
            }
        }
    }

    /// Map value. Passes failure result downstream.
    func map<NewSuccess>(_ transform: @escaping (Success) -> NewSuccess) -> Result<NewSuccess, Failure>.Promise {
        return then { result in
            return result.asConcreteType().map(transform)
        }
    }

    /// Perform action on success.
    func onSuccess(_ onResolve: @escaping (Success) -> Void) -> Result<Success, Failure>.Promise {
        return map { value -> Success in
            onResolve(value)
            return value
        }
    }

    /// Perform action on failure.
    func onFailure(_ onResolve: @escaping (Failure) -> Void) -> Result<Success, Failure>.Promise {
        return mapError { error -> Failure in
            onResolve(error)
            return error
        }
    }

    /// Map value producing Promise. Passes failure result downstream.
    func mapThen<NewSuccess>(_ transform: @escaping (Success) -> Result<NewSuccess, Failure>.Promise) -> Result<NewSuccess, Failure>.Promise {
        return then { result in
            switch result.asConcreteType() {
            case .success(let value):
                return transform(value)
            case .failure(let error):
                return .failure(error)
            }
        }
    }

    /// Map failure. Passes successful result downstream.
    func mapError<NewFailure>(_ transform: @escaping (Failure) -> NewFailure) -> Result<Success, NewFailure>.Promise {
        return then { result in
            return result.asConcreteType().mapError(transform)
        }
    }

    /// Map value to Result. Passes failure result downstream.
    func flatMap<NewSuccess>(_ transform: @escaping (Success) -> Result<NewSuccess, Failure>) -> Result<NewSuccess, Failure>.Promise {
        return then { result in
            return result.asConcreteType().flatMap(transform)
        }
    }

    /// Map failure to Result. Passes successful result downstream.
    func flatMapError<NewFailure>(_ transform: @escaping (Failure) -> Result<Success, NewFailure>) -> Result<Success, NewFailure>.Promise {
        return then { result in
            return result.asConcreteType().flatMapError(transform)
        }
    }
}

extension Promise where Value: AnyResult {
    func tryAwait() throws -> PromiseCompletion<Value.Success> {
        return try self.await().map { result in
            return try result.asConcreteType().get()
        }
    }
}

extension Result {
    func asPromise() -> Result<Success, Failure>.Promise {
        return .resolved(self)
    }

    var error: Failure? {
        switch self {
        case .success:
            return nil
        case .failure(let error):
            return error
        }
    }

    var value: Success? {
        switch self {
        case .success(let value):
            return value
        case .failure:
            return nil
        }
    }
}
