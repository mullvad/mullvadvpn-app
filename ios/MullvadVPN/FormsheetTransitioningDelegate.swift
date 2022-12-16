//
//  FormsheetTransitioningDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 2022-12-16.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class FormsheetTransitioningDelegate: NSObject, UIViewControllerTransitioningDelegate {
    func animationController(
        forPresented presented: UIViewController,
        presenting: UIViewController,
        source: UIViewController
    ) -> UIViewControllerAnimatedTransitioning? {
        return FormsheetPresentationAnimator()
    }

    func animationController(forDismissed dismissed: UIViewController)
        -> UIViewControllerAnimatedTransitioning?
    {
        return FormsheetPresentationAnimator()
    }

    func presentationController(
        forPresented presented: UIViewController,
        presenting: UIViewController?,
        source: UIViewController
    ) -> UIPresentationController? {
        return FormsheetPresentationController(
            presentedViewController: presented,
            presenting: source
        )
    }
}
