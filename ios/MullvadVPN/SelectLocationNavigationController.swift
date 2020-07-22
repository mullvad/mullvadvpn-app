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
    func selectLocationController(_ controller: SelectLocationController, didSelectLocation location: RelayLocation)
    func selectLocationControllerDidCancel(_ controller: SelectLocationController)
}

class SelectLocationNavigationController: UINavigationController {
    private weak var contentController: SelectLocationController?

    weak var selectLocationDelegate: SelectLocationDelegate?

    override init(nibName nibNameOrNil: String?, bundle nibBundleOrNil: Bundle?) {
        super.init(nibName: nil, bundle: nil)
    }

    init() {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        navigationBar.prefersLargeTitles = true
        navigationBar.barStyle = .black
        navigationBar.tintColor = .white

        let contentController = SelectLocationController()
        contentController.navigationItem.title = NSLocalizedString("Select location", comment: "")
        contentController.navigationItem.largeTitleDisplayMode = .always
        contentController.navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDone(_:)))

        contentController.didSelectLocationHandler = { [weak self] (location) in
            guard let self = self, let contentController = self.contentController else { return }

            self.selectLocationDelegate?.selectLocationController(contentController, didSelectLocation: location)
        }

        self.contentController = contentController
        self.viewControllers = [contentController]
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func prefetchData(_ completionHandler: @escaping () -> Void) {
        contentController?.prefetchData(completionHandler: completionHandler)
    }

    @objc func handleDone(_ sender: AnyObject) {
        if let contentController = contentController {
            selectLocationDelegate?.selectLocationControllerDidCancel(contentController)
        }
    }
}
