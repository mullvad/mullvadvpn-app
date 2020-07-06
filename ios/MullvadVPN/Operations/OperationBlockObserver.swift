//
//  OperationBlockObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class OperationBlockObserver<OperationType: OperationProtocol>: OperationObserver {
    private var willFinish: ((OperationType) -> Void)?
    private var didFinish: ((OperationType) -> Void)?

    init(willFinish: ((OperationType) -> Void)? = nil, didFinish: ((OperationType) -> Void)? = nil) {
        self.willFinish = willFinish
        self.didFinish = didFinish
    }

    func operationWillFinish(_ operation: OperationType) {
        self.willFinish?(operation)
    }

    func operationDidFinish(_ operation: OperationType) {
        self.didFinish?(operation)
    }
}

extension OperationProtocol {
    func addDidFinishBlockObserver(_ block: @escaping (Self) -> Void) {
        addObserver(OperationBlockObserver(didFinish: block))
    }
}
