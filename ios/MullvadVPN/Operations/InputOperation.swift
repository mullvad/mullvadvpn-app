//
//  InputOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 09/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol InputOperation: Operation {
    associatedtype Input

    var input: Input? { get }

    func setInputBlock(_ block: @escaping () -> Input?)

    func inject<T>(from dependency: T)
        where T: OutputOperation, T.Output == Input

    func inject<T>(from dependency: T, via block: @escaping (T.Output) -> Input)
        where T: OutputOperation
}

extension InputOperation {
    func inject<T>(from dependency: T) where T: OutputOperation, T.Output == Input {
        inject(from: dependency, via: { $0 })
    }

    func inject<T>(from dependency: T, via block: @escaping (T.Output) -> Input)
        where T: OutputOperation
    {
        setInputBlock {
            return dependency.output.map { value in
                return block(value)
            }
        }
        addDependency(dependency)
    }

    func injectMany<Context>(context: Context) -> InputInjectionBuilder<Self, Context> {
        return InputInjectionBuilder(
            operation: self,
            context: context
        )
    }
}
