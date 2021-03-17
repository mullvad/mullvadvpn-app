//
//  OperationBlockObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class OperationBlockObserver<OperationType: OperationProtocol>: OperationObserver {
    private var willExecute: ((OperationType) -> Void)?
    private var willFinish: ((OperationType) -> Void)?
    private var didFinish: ((OperationType) -> Void)?

    let queue: DispatchQueue?

    init(
        queue: DispatchQueue? = nil,
        willExecute: ((OperationType) -> Void)? = nil,
        willFinish: ((OperationType) -> Void)? = nil,
        didFinish: ((OperationType) -> Void)? = nil
    ) {
        self.queue = queue
        self.willExecute = willExecute
        self.willFinish = willFinish
        self.didFinish = didFinish
    }

    func operationWillExecute(_ operation: OperationType) {
        if let willExecute = willExecute {
            scheduleEvent {
                willExecute(operation)
            }
        }
    }

    func operationWillFinish(_ operation: OperationType) {
        if let willFinish = self.willFinish {
            scheduleEvent {
                willFinish(operation)
            }
        }
    }

    func operationDidFinish(_ operation: OperationType) {
        if let didFinish = self.didFinish {
            scheduleEvent {
                didFinish(operation)
            }
        }
    }

    private func scheduleEvent(_ body: @escaping () -> Void) {
        if let queue = queue {
            queue.async(execute: body)
        } else {
            body()
        }
    }
}

extension OperationProtocol {
    func addDidFinishBlockObserver(queue: DispatchQueue? = nil, _ block: @escaping (Self) -> Void) {
        addObserver(OperationBlockObserver(queue: queue, didFinish: block))
    }
}
