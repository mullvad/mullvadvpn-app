// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

/**
 A struct holding modal presentation configuration.
 */
@MainActor
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
