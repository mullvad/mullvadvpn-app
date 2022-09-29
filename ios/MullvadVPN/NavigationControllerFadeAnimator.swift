//
//  NavigationControllerFadeAnimator.swift
//  MullvadVPN
//
//  Created by pronebird on 28/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class NavigationControllerFadeAnimator: NSObject, UIViewControllerAnimatedTransitioning {
    func transitionDuration(using transitionContext: UIViewControllerContextTransitioning?)
        -> TimeInterval
    {
        return (transitionContext?.isAnimated ?? true) ? 0.3 : 0
    }

    func animateTransition(using transitionContext: UIViewControllerContextTransitioning) {
        guard let toViewController = transitionContext
            .viewController(forKey: UITransitionContextViewControllerKey.to)
        else {
            return
        }

        transitionContext.containerView.addSubview(toViewController.view)
        toViewController.view.frame = transitionContext.finalFrame(for: toViewController)
        toViewController.view.alpha = 0

        UIView.animate(
            withDuration: transitionDuration(using: transitionContext),
            animations: {
                toViewController.view.alpha = 1
            },
            completion: { finished in
                transitionContext.completeTransition(
                    !transitionContext.transitionWasCancelled
                )
            }
        )
    }
}
