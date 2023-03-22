//
//  ModalPresentationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 14/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 A struct holding modal presentation configuration.
 */
struct ModalPresentationConfiguration {
    var preferredContentSize: CGSize?
    var modalPresentationStyle: UIModalPresentationStyle?
    var isModalInPresentation: Bool?
    var transitioningDelegate: UIViewControllerTransitioningDelegate?
    var presentationControllerDelegate: UIAdaptivePresentationControllerDelegate?

    func apply(to vc: UIViewController) {
        vc.transitioningDelegate = transitioningDelegate

        if let modalPresentationStyle = modalPresentationStyle {
            vc.modalPresentationStyle = modalPresentationStyle
        }

        if let preferredContentSize = preferredContentSize {
            vc.preferredContentSize = preferredContentSize
        }

        if let isModalInPresentation = isModalInPresentation {
            vc.isModalInPresentation = isModalInPresentation
        }

        vc.presentationController?.delegate = presentationControllerDelegate
    }

    /**
     Wraps `presentationControllerDelegate` into forwarding delegate that intercepts interactive
     dismissal and calls `dismissalHandler` while proxying all delegate calls to the former
     delegate.
     */
    mutating func notifyInteractiveDismissal(_ dismissalHandler: @escaping () -> Void) {
        presentationControllerDelegate =
            PresentationControllerDismissalInterceptor(
                forwardingTarget: presentationControllerDelegate
            ) { _ in
                dismissalHandler()
            }
    }
}
