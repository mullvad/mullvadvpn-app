//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class AlertPresenter {
    static let alertControllerDidDismissNotification = Notification.Name("UIAlertControllerDidDismiss")

    private let operationQueue: OperationQueue = {
        let operationQueue = OperationQueue()
        operationQueue.name = "AlertPresenterQueue"
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    private static let initClass: Void = {
        /// Swizzle `viewDidDisappear` on `UIAlertController` in order to be able to
        /// detect when the controller disappears.
        /// The event is broadcasted via `AlertPresenter.alertControllerDidDismissNotification` notification.
        swizzleMethod(aClass: UIAlertController.self, originalSelector: #selector(UIAlertController.viewDidDisappear(_:)), newSelector: #selector(UIAlertController.alertPresenter_viewDidDisappear(_:)))
    }()

    init() {
        _ = Self.initClass
    }

    func enqueue(_ alertController: UIAlertController, presentingController: UIViewController, presentCompletion: (() -> Void)? = nil) {
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

fileprivate extension UIAlertController {
    @objc dynamic func alertPresenter_viewDidDisappear(_ animated: Bool) {
        // Call super implementation
        alertPresenter_viewDidDisappear(animated)

        if presentingViewController == nil {
            NotificationCenter.default.post(name: AlertPresenter.alertControllerDidDismissNotification, object: self)
        }
    }
}
