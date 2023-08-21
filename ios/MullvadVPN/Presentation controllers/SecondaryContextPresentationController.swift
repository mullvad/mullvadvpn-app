//
//  SecondaryContextPresentationController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 This is a presentation controller class used for presentation of secondary navigation context
 in application coordinator.
 */
class SecondaryContextPresentationController: FormSheetPresentationController {
    override func presentationTransitionWillBegin() {
        super.presentationTransitionWillBegin()

        updateHeaderBarHidden()

        if let containerView,
           let rootContainer = presentingViewController as? RootContainerViewController {
            rootContainer.addTrailingButtonsToPresentationContainer(containerView)
        }
    }

    override func dismissalTransitionDidEnd(_ completed: Bool) {
        super.dismissalTransitionDidEnd(completed)

        if let rootContainer = presentingViewController as? RootContainerViewController, completed {
            rootContainer.removeTrailingButtonsFromPresentationContainer()
        }
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        updateHeaderBarHidden()
    }

    private func updateHeaderBarHidden() {
        let presentedController = presentedViewController as? RootContainerViewController

        presentedController?.setOverrideHeaderBarHidden(
            isInFullScreenPresentation ? nil : true,
            animated: false
        )
    }
}

class SecondaryContextTransitioningDelegate: FormSheetTransitioningDelegate {
    convenience init(adjustViewWhenKeyboardAppears: Bool) {
        let option = FormSheetPresentationOptions(
            useFullScreenPresentationInCompactWidth: true,
            adjustViewWhenKeyboardAppears: adjustViewWhenKeyboardAppears
        )
        self.init(options: option)
    }

    private override init(options: FormSheetPresentationOptions) {
        super.init(options: options)
    }

    override func presentationController(
        forPresented presented: UIViewController,
        presenting: UIViewController?,
        source: UIViewController
    ) -> UIPresentationController? {
        let presentationController = SecondaryContextPresentationController(
            presentedViewController: presented,
            presenting: source,
            options: options
        )

        return presentationController
    }
}
