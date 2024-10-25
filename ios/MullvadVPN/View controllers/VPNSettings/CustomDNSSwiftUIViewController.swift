//
//  CustomDNSSwiftUIViewController.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

import Foundation
import SwiftUI
import UIKit

class CustomDNSSwiftUIViewController: UIViewController {
    private let interactor: VPNSettingsInteractor
    private let alertPresenter: AlertPresenter

    init(interactor: VPNSettingsInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

        super.init(nibName: nil, bundle: nil)
    }

    override func viewDidLoad() {
        let rootView = CustomDNSSwiftUIView(viewModel: self.interactor.tunnelSettings.dnsSettings.viewModel())

        let hostingController = UIHostingController(rootView: rootView)

        addChild(hostingController)

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "VPNSettings",
            value: "DNS settings",
            comment: ""
        )

        navigationItem.rightBarButtonItem = editButtonItem
        navigationItem.rightBarButtonItem?.accessibilityIdentifier = .dnsSettingsEditButton

        view.addConstrainedSubviews([hostingController.view]) {
            hostingController.view.pinEdgesToSuperview()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
