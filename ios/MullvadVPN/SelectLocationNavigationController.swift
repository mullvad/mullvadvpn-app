//
//  SelectLocationNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 22/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

protocol SelectLocationDelegate: class {
    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectLocation location: RelayLocation)
    func selectLocationViewControllerDidCancel(_ controller: SelectLocationViewController)
}

class SelectLocationNavigationController: UINavigationController {
    private weak var contentController: SelectLocationViewController?

    weak var selectLocationDelegate: SelectLocationDelegate?

    init() {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        navigationBar.prefersLargeTitles = true
        navigationBar.barStyle = .black
        navigationBar.tintColor = .white

        let contentController = SelectLocationViewController()
        contentController.navigationItem.title = NSLocalizedString("Select location", comment: "")
        contentController.navigationItem.largeTitleDisplayMode = .always
        contentController.navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDone(_:)))

        contentController.didSelectLocationHandler = { [weak self] (location) in
            guard let self = self, let contentController = self.contentController else { return }

            self.selectLocationDelegate?.selectLocationViewController(contentController, didSelectLocation: location)
        }

        self.contentController = contentController
        self.viewControllers = [contentController]
    }

    override init(nibName nibNameOrNil: String?, bundle nibBundleOrNil: Bundle?) {
        // This override has to exist to prevent crash on iOS 12 where `UINavigationController`
        // calls `self.init(nibName:bundle:)` internally.
        super.init(nibName: nibNameOrNil, bundle: nibBundleOrNil)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func prefetchData(_ completionHandler: @escaping () -> Void) {
        contentController?.prefetchData(completionHandler: completionHandler)
    }

    @objc func handleDone(_ sender: AnyObject) {
        if let contentController = contentController {
            selectLocationDelegate?.selectLocationViewControllerDidCancel(contentController)
        }
    }
}
