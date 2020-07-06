//
//  TransformOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TransformOperation<Input, Output>: AsyncOperation, InputOperation, OutputOperation {
    private enum Executor {
        case callback((Input, @escaping (Output) -> Void) -> Void)
        case transform((Input) -> Output)
    }

    private let executor: Executor

    private init(input: Input? = nil, executor: Executor) {
        self.executor = executor

        super.init()
        self.input = input
    }

    convenience init(input: Input? = nil, _ block: @escaping (Input, @escaping (Output) -> Void) -> Void) {
        self.init(input: input, executor: .callback(block))
    }

    convenience init(input: Input? = nil, _ block: @escaping (Input) -> Output) {
        self.init(input: input, executor: .transform(block))
    }

    override func main() {
        guard let input = input else {
            self.finish()
            return
        }

        switch executor {
        case .callback(let block):
            block(input) { [weak self] (result) in
                self?.finish(with: result)
            }

        case .transform(let block):
            self.finish(with: block(input))
        }
    }
}
