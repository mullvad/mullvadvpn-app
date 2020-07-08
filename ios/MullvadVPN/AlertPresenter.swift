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

    init() {
        _ = AlertPresenterUIKitHooks.once
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

/// A helper struct that swizzles `viewDidDisappear` on `UIAlertController` in order to be able to
/// detect when the controller disappears.
/// The event is broadcasted via `kUIAlertControllerDidDissmissNotification` notification.
private struct AlertPresenterUIKitHooks {
    typealias MethodType = @convention(c) (UIAlertController, Selector, Bool) -> Void
    typealias BlockImpType = @convention(block) (UIAlertController, Bool) -> Void

    static let once = AlertPresenterUIKitHooks()

    private init() {
        let originalSelector = #selector(UIAlertController.viewDidDisappear(_:))
        let originalMethod = class_getInstanceMethod(UIAlertController.self, originalSelector)!

        var originalIMP: IMP? = nil
        let swizzledBlockIMP: BlockImpType = { (receiver, animated) in
            let superIMP = originalIMP.map { unsafeBitCast($0, to: MethodType.self) }
            superIMP?(receiver, originalSelector, animated)
            
            Self.handleViewDidDisappear(receiver, animated)
        }

        let swizzledIMP = imp_implementationWithBlock(unsafeBitCast(swizzledBlockIMP, to: AnyObject.self))
        originalIMP = method_setImplementation(originalMethod, swizzledIMP)
    }

    private static func handleViewDidDisappear(_ alertController: UIAlertController, _ animated: Bool) {
        if alertController.presentingViewController == nil {
            NotificationCenter.default.post(name: kUIAlertControllerDidDissmissNotification, object: alertController)
        }
    }
}
