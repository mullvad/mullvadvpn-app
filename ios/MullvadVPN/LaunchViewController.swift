//
//  LaunchViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LaunchViewController: UIViewController {
    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init() {
        super.init(nibName: nil, bundle: nil)

        let storyboard = UIStoryboard(name: "LaunchScreen", bundle: nil)

        guard let initialController = storyboard.instantiateInitialViewController() else { return }

        initialController.view.translatesAutoresizingMaskIntoConstraints = false

        addChild(initialController)
        view.addSubview(initialController.view)
        initialController.didMove(toParent: self)

        NSLayoutConstraint.activate([
            initialController.view.topAnchor.constraint(equalTo: view.topAnchor),
            initialController.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            initialController.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            initialController.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
