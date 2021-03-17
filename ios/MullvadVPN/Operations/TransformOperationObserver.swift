//
//  TransformOperationObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A private type erasing observer that type casts the input operation type to the expected
/// operation type before calling the wrapped observer
class TransformOperationObserver<S: OperationProtocol>: OperationObserver {
    private let willExecute: (S) -> Void
    private let willFinish: (S) -> Void
    private let didFinish: (S) -> Void

    init<T: OperationObserver>(_ observer: T) {
        willExecute = Self.wrap(observer.operationWillExecute)
        willFinish = Self.wrap(observer.operationWillFinish)
        didFinish = Self.wrap(observer.operationDidFinish)
    }

    func operationWillExecute(_ operation: S) {
        willExecute(operation)
    }

    func operationWillFinish(_ operation: S) {
        willFinish(operation)
    }

    func operationDidFinish(_ operation: S) {
        didFinish(operation)
    }

    private class func wrap<U>(_ body: @escaping (U) -> Void) -> (S) -> Void {
        return { (operation: S) in
            if let transformed = operation as? U {
                body(transformed)
            } else {
                fatalError("\(Self.self) failed to cast \(S.self) to \(U.self)")
            }
        }
    }
}
