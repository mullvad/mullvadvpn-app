//
//  WelcomeViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit
import UIKit

protocol WelcomeViewControllerDelegate: AnyObject {
    func didRequestToShowInfo(controller: WelcomeViewController)
    func didRequestToViewPurchaseOptions(
        accountNumber: String
    )
}

class WelcomeViewController: UIViewController, RootContainment {
    private lazy var contentView: WelcomeContentView = {
        let view = WelcomeContentView()
        view.delegate = self
        return view
    }()

    private let interactor: WelcomeInteractor

    weak var delegate: WelcomeViewControllerDelegate?

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        HeaderBarPresentation(style: .default, showsDivider: true)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    var prefersDeviceInfoBarHidden: Bool {
        true
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(interactor: WelcomeInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        contentView.viewModel = interactor.viewModel
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        interactor.viewWillAppear = true
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        interactor.viewDidDisappear = true
    }

    private func configureUI() {
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
        }
    }
}

extension WelcomeViewController: @preconcurrency WelcomeContentViewDelegate {
    func didTapInfoButton(welcomeContentView: WelcomeContentView, button: UIButton) {
        delegate?.didRequestToShowInfo(controller: self)
    }

    func didTapPurchaseButton(welcomeContentView: WelcomeContentView, button: AppButton) {
        delegate?.didRequestToViewPurchaseOptions(accountNumber: interactor.accountNumber)
    }

    func didTapCopyButton(welcomeContentView: WelcomeContentView, button: UIButton) {
        UIPasteboard.general.string = interactor.accountNumber
    }
}
