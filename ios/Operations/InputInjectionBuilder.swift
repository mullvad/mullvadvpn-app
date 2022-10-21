//
//  InputInjectionBuilder.swift
//  Operations
//
//  Created by pronebird on 09/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol OperationInputContext {
    associatedtype Input

    func reduce() -> Input?
}

public final class InputInjectionBuilder<OperationType, Context>
    where OperationType: InputOperation
{
    public typealias InputBlock = (inout Context) -> Void

    private let operation: OperationType
    private var context: Context
    private var inputBlocks: [InputBlock] = []

    public init(operation: OperationType, context: Context) {
        self.operation = operation
        self.context = context
    }

    public func inject<T>(
        from dependency: T,
        assignOutputTo keyPath: WritableKeyPath<Context, T.Output?>
    ) -> Self
        where T: OutputOperation
    {
        return inject(from: dependency) { context, output in
            context[keyPath: keyPath] = output
        }
    }

    public func inject<T>(
        from dependency: T,
        via block: @escaping (inout Context, T.Output) -> Void
    ) -> Self
        where T: OutputOperation
    {
        inputBlocks.append { context in
            if let output = dependency.output {
                block(&context, output)
            }
        }

        operation.addDependency(dependency)

        return self
    }

    public func injectCompletion<T, Success, Failure>(
        from dependency: T,
        via block: @escaping (inout Context, T.Completion) -> Void
    ) -> Self
        where T: ResultOperation<Success, Failure>
    {
        inputBlocks.append { context in
            if let completion = dependency.completion {
                block(&context, completion)
            }
        }

        operation.addDependency(dependency)

        return self
    }

    public func reduce(_ reduceBlock: @escaping (Context) -> OperationType.Input?) {
        operation.setInputBlock {
            for inputBlock in self.inputBlocks {
                inputBlock(&self.context)
            }

            return reduceBlock(self.context)
        }
    }
}

extension InputInjectionBuilder
    where Context: OperationInputContext,
    Context.Input == OperationType.Input
{
    public func reduce() {
        reduce { context in
            return context.reduce()
        }
    }
}
