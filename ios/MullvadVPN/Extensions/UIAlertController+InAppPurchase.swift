//
//  UIAlertController+InAppPurchase.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit
import UIKit

extension UIAlertController {
    public static func showInAppPurchaseAlert(
        products: [SKProduct],
        didRequestPurchase: @escaping (SKProduct) -> Void
    ) -> UIAlertController {
        let localizedString = NSLocalizedString(
            "ADD_TIME_BUTTON",
            tableName: "Welcome",
            value: "Add Time",
            comment: ""
        )
        let alert = UIAlertController(title: localizedString, message: nil, preferredStyle: .actionSheet)
        products.sortedByPrice().forEach { product in
            guard let localizedTitle = product.customLocalizedTitle else {
                return
            }
            let action = UIAlertAction(title: localizedTitle, style: .default, handler: { _ in
                alert.dismiss(animated: true, completion: {
                    didRequestPurchase(product)
                })
            })
            action
                .accessibilityIdentifier =
                "\(AccessibilityIdentifier.purchaseButton.asString)_\(product.productIdentifier)"
            alert.addAction(action)
        }
        let cancelAction = UIAlertAction(title: NSLocalizedString(
            "PRODUCT_LIST_CANCEL_BUTTON",
            tableName: "Welcome",
            value: "Cancel",
            comment: ""
        ), style: .cancel)
        cancelAction.accessibilityIdentifier = AccessibilityIdentifier.cancelPurchaseListButton.asString
        alert.addAction(cancelAction)
        return alert
    }
}
