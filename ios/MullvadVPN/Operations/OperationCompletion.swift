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
}
