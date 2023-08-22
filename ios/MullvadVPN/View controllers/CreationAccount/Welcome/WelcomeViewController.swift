//
//  WelcomeViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit
protocol WelcomeViewControllerDelegate: AnyObject {
    func didRequestToPurchaseCredit(controller: WelcomeViewController)
    func didRequestToRedeemVoucher(controller: WelcomeViewController)
    func didRequestToShowInfo(controller: WelcomeViewController)
    func didUpdateDeviceState(deviceState: DeviceState)
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

    var prefersNotificationBarHidden: Bool {
        true
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

        interactor.didUpdateDeviceState = { [weak self] deviceState in
            self?.delegate?.didUpdateDeviceState(deviceState: deviceState)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        interactor.startAccountUpdateTimer()
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        interactor.stopAccountUpdateTimer()
    }

    private func configureUI() {
        view.addSubview(contentView)
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
        }
    }
}

extension WelcomeViewController: WelcomeContentViewDelegate {
    func didTapInfoButton(welcomeContentView: WelcomeContentView, button: UIButton) {
        delegate?.didRequestToShowInfo(controller: self)
    }

    func didTapPurchaseButton(welcomeContentView: WelcomeContentView, button: AppButton) {
        delegate?.didRequestToPurchaseCredit(controller: self)
    }

    func didTapRedeemVoucherButton(welcomeContentView: WelcomeContentView, button: AppButton) {
        delegate?.didRequestToRedeemVoucher(controller: self)
    }
}
