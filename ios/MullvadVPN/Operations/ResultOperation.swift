//
//  ResultOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ResultOperation<Success, Failure: Error>: AsyncOperation, OutputOperation {
    typealias Output = Result<Success, Failure>

    private enum Executor {
        case callback((@escaping (Result<Success, Failure>) -> Void) -> Void)
        case transform(() -> Result<Success, Failure>)
    }

    private let executor: Executor

    private init(_ executor: Executor) {
        self.executor = executor
    }

    convenience init(_ block: @escaping (@escaping (Output) -> Void) -> Void) {
        self.init(.callback(block))
    }

    convenience init(_ block: @escaping () -> Output) {
        self.init(.transform(block))
    }

    override func main() {
        switch executor {
        case .callback(let block):
            block { [weak self] (result) in
                self?.finish(with: result)
            }

        case .transform(let block):
            self.finish(with: block())
        }
    }

}

extension ResultOperation where Failure == Never {
    /// A convenience initializer for infallible `ResultOperation` that automatically wraps the
    /// return value of the given closure into `Result<Success, Never>`
    convenience init(_ block: @escaping () -> Success) {
        self.init(.transform({ .success(block()) }))
    }

    /// A convenience initializer for infallible `ResultOperation` that automatically wraps the
    /// value, passed to the given closure, into `Result<Success, Never>`
    convenience init(_ block: @escaping (@escaping (Success) -> Void) -> Void) {
        self.init(.callback({ (finish) in
            block {
                finish(.success($0))
            }
        }))
    }
}
