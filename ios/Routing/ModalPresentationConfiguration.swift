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
public struct ModalPresentationConfiguration {
    var preferredContentSize: CGSize?
    var modalPresentationStyle: UIModalPresentationStyle?
    var modalTransitionStyle: UIModalTransitionStyle?
    var isModalInPresentation: Bool?
    var transitioningDelegate: UIViewControllerTransitioningDelegate?
    var presentationControllerDelegate: UIAdaptivePresentationControllerDelegate?

    public init(
        preferredContentSize: CGSize? = nil,
        modalPresentationStyle: UIModalPresentationStyle? = nil,
        modalTransitionStyle: UIModalTransitionStyle? = nil,
        isModalInPresentation: Bool? = nil,
        transitioningDelegate: UIViewControllerTransitioningDelegate? = nil,
        presentationControllerDelegate: UIAdaptivePresentationControllerDelegate? = nil
    ) {
        self.preferredContentSize = preferredContentSize
        self.modalPresentationStyle = modalPresentationStyle
        self.modalTransitionStyle = modalTransitionStyle
        self.isModalInPresentation = isModalInPresentation
        self.transitioningDelegate = transitioningDelegate
        self.presentationControllerDelegate = presentationControllerDelegate
    }

    public func apply(to vc: UIViewController) {
        vc.transitioningDelegate = transitioningDelegate

        if let modalPresentationStyle {
            vc.modalPresentationStyle = modalPresentationStyle
        }

        if let modalTransitionStyle {
            vc.modalTransitionStyle = modalTransitionStyle
        }

        if let preferredContentSize {
            vc.preferredContentSize = preferredContentSize
        }

        if let isModalInPresentation {
            vc.isModalInPresentation = isModalInPresentation
        }

        vc.presentationController?.delegate = presentationControllerDelegate
    }

    /**
     Wraps `presentationControllerDelegate` into forwarding delegate that intercepts interactive
     dismissal and calls `dismissalHandler` while proxying all delegate calls to the former
     delegate.
     */
    public mutating func notifyInteractiveDismissal(_ dismissalHandler: @escaping () -> Void) {
        presentationControllerDelegate =
            PresentationControllerDismissalInterceptor(
                forwardingTarget: presentationControllerDelegate
            ) { _ in
                dismissalHandler()
            }
    }
}
