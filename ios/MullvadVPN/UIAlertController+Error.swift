//
//  UIAlertController+Error.swift
//  MullvadVPN
//
//  Created by pronebird on 11/12/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import UIKit

/// An extension for presenting `LocalizedError` subclasses in `UIAlertController`
extension UIAlertController {

    convenience init<Error>(_ error: Error, preferredStyle: UIAlertController.Style)
        where Error: LocalizedError
    {
        let title = error.errorDescription
        let message = [error.failureReason, error.recoverySuggestion]
            .compactMap { $0 }
            .joined(separator: "\n\n")

        self.init(title: title, message: message, preferredStyle: preferredStyle)
    }

}

extension UIViewController {

    /// Present an instance of `LocalizedError` using `UIAlertController`
    /// Note: this method adds a default "OK" action when `configurationBlock` is not given
    func presentError<Error>(
        _ error: Error,
        preferredStyle: UIAlertController.Style,
        configurationBlock: ((UIAlertController) -> Void)? = nil,
        completionBlock: (() -> Void)? = nil)
        where Error: LocalizedError
    {
        let alertController = UIAlertController(error, preferredStyle: preferredStyle)

        if let configurationBlock = configurationBlock {
            configurationBlock(alertController)
        } else {
            alertController.addAction(
                UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
            )
        }

        self.present(alertController, animated: true, completion: completionBlock)
    }

}
