//
//  WelcomeViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import StoreKit
import UIKit

protocol WelcomeViewControllerDelegate: AnyObject {
    func didRequestToRedeemVoucher(controller: WelcomeViewController)
    func didRequestToShowInfo(controller: WelcomeViewController)
    func didRequestToDoInAppPurchase(controller: WelcomeViewController, accountNumber: String, product: SKProduct)
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
        interactor.didChangeInAppPurchaseState = { [weak self] interactor, productState in
            self?.contentView.productState = productState
        }
        interactor.viewDidLoad = true
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

    func didTapPurchaseButton(welcomeContentView: WelcomeContentView, button: AppButton, product: SKProduct) {
        delegate?.didRequestToDoInAppPurchase(
            controller: self,
            accountNumber: interactor.accountNumber,
            product: product
        )
    }

    func didTapRedeemVoucherButton(welcomeContentView: WelcomeContentView, button: AppButton) {
        delegate?.didRequestToRedeemVoucher(controller: self)
    }
}

extension WelcomeViewController: InAppPurchaseViewControllerDelegate {
    func didBeginPayment() {
        contentView.isPurchasing = true
    }

    func didEndPayment() {
        contentView.isPurchasing = false
    }
}
