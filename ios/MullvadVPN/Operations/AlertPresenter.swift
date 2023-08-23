//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Operations
import UIKit

final class AlertPresenter {
    private let operationQueue = AsyncOperationQueue.makeSerial()

    func enqueue(
        _ alertController: AlertViewController,
        presentingController: UIViewController,
        presentCompletion: (() -> Void)? = nil
    ) {
        let operation = PresentAlertOperation(
            alertController: alertController,
            presentingController: presentingController,
            presentCompletion: presentCompletion
        )

        operationQueue.addOperation(operation)
    }

    func cancelAll() {
        operationQueue.cancelAllOperations()
    }
}
