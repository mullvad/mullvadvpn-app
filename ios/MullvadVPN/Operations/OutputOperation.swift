//
//  OutputOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol OutputOperation: OperationProtocol {
    associatedtype Output

    var output: Output? { get set }

    func finish(with output: Output)
}

extension OutputOperation {
    func finish(with output: Output) {
        self.output = output
        self.finish()
    }
}

private var kOutputOperationAssociatedValue = 0
extension OutputOperation where Self: OperationSubclassing {
    var output: Output? {
        get {
            return synchronized {
                return AssociatedValue.get(object: self, key: &kOutputOperationAssociatedValue)
            }
        }
        set {
            synchronized {
                AssociatedValue.set(object: self, key: &kOutputOperationAssociatedValue, value: newValue)
            }
        }
    }
}

extension OperationProtocol where Self: OutputOperation {
    func addDidFinishBlockObserver(queue: DispatchQueue? = nil, _ block: @escaping (Self, Output) -> Void) {
        addDidFinishBlockObserver(queue: queue) { (operation) in
            if let output = operation.output {
                block(operation, output)
            }
        }
    }
}
