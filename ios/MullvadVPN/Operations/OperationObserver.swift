//
//  OperationObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol OperationObserver {
    func didAttach(to operation: Operation)
    func operationDidStart(_ operation: Operation)
    func operationDidCancel(_ operation: Operation)
    func operationDidFinish(_ operation: Operation, error: Error?)
}

/// Block based operation observer.
class OperationBlockObserver<OperationType: Operation>: OperationObserver {
    typealias VoidBlock = (OperationType) -> Void
    typealias FinishBlock = (OperationType, Error?) -> Void

    private let _didAttach: VoidBlock?
    private let _didStart: VoidBlock?
    private let _didCancel: VoidBlock?
    private let _didFinish: FinishBlock?

    init(
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

    func didAttach(to operation: Operation) {
        if let operation = operation as? OperationType {
            _didAttach?(operation)
        }
    }

    func operationDidStart(_ operation: Operation) {
        if let operation = operation as? OperationType {
            _didStart?(operation)
        }
    }

    func operationDidCancel(_ operation: Operation) {
        if let operation = operation as? OperationType {
            _didCancel?(operation)
        }
    }

    func operationDidFinish(_ operation: Operation, error: Error?) {
        if let operation = operation as? OperationType {
            _didFinish?(operation, error)
        }
    }
}
