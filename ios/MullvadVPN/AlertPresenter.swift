//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

private let kUIAlertControllerDidDissmissNotification = Notification.Name("UIAlertControllerDidDismiss")

class AlertPresenter {
    private enum ExclusivityCategory {
        case exclusive
    }

    private let operationQueue = OperationQueue()
    private lazy var exclusivityController = ExclusivityController<ExclusivityCategory>(operationQueue: operationQueue)

    private static let initClass: Void = {
        /// Swizzle `viewDidDisappear` on `UIAlertController` in order to be able to
        /// detect when the controller disappears.
        /// The event is broadcasted via `kUIAlertControllerDidDissmissNotification` notification.
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

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

}


fileprivate extension UIAlertController {
    @objc dynamic func alertPresenter_viewDidDisappear(_ animated: Bool) {
        // Call super implementation
        alertPresenter_viewDidDisappear(animated)

        if presentingViewController == nil {
            NotificationCenter.default.post(name: kUIAlertControllerDidDissmissNotification, object: self)
        }
    }
}


private class PresentAlertOperation: AsyncOperation {
    private let alertController: UIAlertController
    private let presentingController: UIViewController
    private var dismissalObserver: NSObjectProtocol?
    private let presentCompletion: (() -> Void)?

    init(alertController: UIAlertController, presentingController: UIViewController, presentCompletion: (() -> Void)? = nil) {
        self.alertController = alertController
        self.presentingController = presentingController
        self.presentCompletion = presentCompletion

        super.init()
    }

    override func main() {
        DispatchQueue.main.async {
            self.dismissalObserver = NotificationCenter.default.addObserver(
                forName: kUIAlertControllerDidDissmissNotification,
                object: self.alertController,
                queue: nil,
                using: { [weak self] (note) in
                    self?.finish()
            })

            self.presentingController.present(self.alertController, animated: true, completion: self.presentCompletion)
        }
    }
}
