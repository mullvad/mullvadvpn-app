//
//  InputOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol InputOperation: OperationProtocol {
    associatedtype Input

    /// When overriding `input` in Subclasses, make sure to call `operationDidSetInput`
    var input: Input? { get set }

    func operationDidSetInput(_ input: Input?)
}

private var kInputOperationAssociatedValue = 0
extension InputOperation where Self: OperationSubclassing {
    var input: Input? {
        get {
            return synchronized {
                return AssociatedValue.get(object: self, key: &kInputOperationAssociatedValue)
            }
        }
        set {
            synchronized {
                AssociatedValue.set(object: self, key: &kInputOperationAssociatedValue, value: newValue)

                operationDidSetInput(newValue)
            }
        }
    }

    func operationDidSetInput(_ input: Input?) {
        // Override in subclasses
    }
}

extension InputOperation {

    @discardableResult func inject<Dependency>(from dependency: Dependency, via block: @escaping (Dependency.Output) -> Input?) -> Self
        where Dependency: OutputOperation
    {
        let observer = OperationBlockObserver<Dependency>(willFinish: { [weak self] (operation) in
            guard let self = self else { return }

            if let output = operation.output {
                self.input = block(output)
            }
        })
        dependency.addObserver(observer)
        addDependency(dependency)

        return self
    }

    @discardableResult func injectResult<Dependency>(from dependency: Dependency) -> Self
        where Dependency: OutputOperation, Dependency.Output == Input?
    {
        return self.inject(from: dependency, via: { $0 })
    }

    /// Inject input from operation that outputs `Result<Input, Failure>`
    @discardableResult func injectResult<Dependency, Failure>(from dependency: Dependency) -> Self
        where Dependency: OutputOperation, Failure: Error, Dependency.Output == Result<Input, Failure>
    {
        return self.inject(from: dependency) { (output) -> Input? in
            switch output {
            case .success(let value):
                return value
            case .failure:
                return nil
            }
        }
    }

    /// Inject input from operation that outputs `Result<Input, Never>`
    @discardableResult func injectResult<Dependency>(from dependency: Dependency) -> Self
        where Dependency: OutputOperation, Dependency.Output == Result<Input, Never>
    {
        return self.inject(from: dependency) { (output) -> Input? in
            switch output {
            case .success(let value):
                return value
            }
        }
    }

    /// Inject input from operation that outputs `Result<Input?, Never>`
    @discardableResult func injectResult<Dependency>(from dependency: Dependency) -> Self
        where Dependency: OutputOperation, Dependency.Output == Result<Input?, Never>
    {
        return self.inject(from: dependency) { (output) -> Input? in
            switch output {
            case .success(let value):
                return value
            }
        }
    }
}
