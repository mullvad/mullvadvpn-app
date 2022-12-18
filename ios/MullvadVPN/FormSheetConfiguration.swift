//
//  FormSheetConfiguration.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct FormSheetConfiguration {
    var transitioningDelegate: UIViewControllerTransitioningDelegate
    var modalPresentationStyle: UIModalPresentationStyle
    var preferredContentSize: CGSize
    var presentationControllerDelegate: UIAdaptivePresentationControllerDelegate
    var isModalInPresentation: Bool
    var popoverPresentationControllerSourceView: UIView?

    func apply(to viewController: UIViewController) {
        viewController.transitioningDelegate = transitioningDelegate
        viewController.modalPresentationStyle = modalPresentationStyle
        viewController.preferredContentSize = preferredContentSize
        viewController.presentationController?.delegate = presentationControllerDelegate
        viewController.isModalInPresentation = isModalInPresentation
        viewController.popoverPresentationController?.sourceView = popoverPresentationControllerSourceView
    }
}
