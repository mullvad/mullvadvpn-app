//
//  OperationObserver.swift
//  Operations
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol OperationObserver {
    func didAttach(to operation: Operation)
    func operationDidStart(_ operation: Operation)
    func operationDidCancel(_ operation: Operation)
    func operationDidFinish(_ operation: Operation, error: Error?)
}

/// Block based operation observer.
public class OperationBlockObserver<OperationType: Operation>: OperationObserver {
    public typealias VoidBlock = (OperationType) -> Void
    public typealias FinishBlock = (OperationType, Error?) -> Void

    private let _didAttach: VoidBlock?
    private let _didStart: VoidBlock?
    private let _didCancel: VoidBlock?
    private let _didFinish: FinishBlock?

    public init(
        didAttach: VoidBlock? = nil,
        didStart: VoidBlock? = nil,
        didCancel: VoidBlock? = nil,
        didFinish: FinishBlock? = nil
    ) {
        _didAttach = didAttach
        _didStart = didStart
        _didCancel = didCancel
        _didFinish = didFinish
    }

    public func didAttach(to operation: Operation) {
        if let operation = operation as? OperationType {
            _didAttach?(operation)
        }
    }

    public func operationDidStart(_ operation: Operation) {
        if let operation = operation as? OperationType {
            _didStart?(operation)
        }
    }

    public func operationDidCancel(_ operation: Operation) {
        if let operation = operation as? OperationType {
            _didCancel?(operation)
        }
    }

    public func operationDidFinish(_ operation: Operation, error: Error?) {
        if let operation = operation as? OperationType {
            _didFinish?(operation, error)
        }
    }
}
