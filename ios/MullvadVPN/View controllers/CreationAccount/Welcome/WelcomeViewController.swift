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
//    func didRequestToPurchaseCredit(controller: WelcomeViewController, accountNumber: String, product: SKProduct)
    func didRequestToViewPurchaseOptions(controller: WelcomeViewController, availableProducts: [SKProduct], accountNumber: String)
    func didRequestToShowFailToFetchProducts(controller: WelcomeViewController)
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
//        interactor.didChangeInAppPurchaseState = { [weak self] productState in
//            guard let self else { return }
//            switch productState {
//            case .received(let products):
//                delegate?.didRequestToViewPurchaseOptions(controller: self, availableProducts: products, accountNumber: interactor.accountNumber)
//            case .failed:
//                delegate?.didRequestToShowFailToFetchProducts(controller: self)
//            default: break}
//         
//            self.contentView.productState = productState
//        }
        interactor.viewDidLoad = true
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
        let productIdentifiers = Set(StoreSubscription.allCases)
        contentView.isFetchingProducts = true
        _ = interactor.requestProducts(with: productIdentifiers) { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let success):
                let products = success.products
                if !products.isEmpty {
                    delegate?.didRequestToViewPurchaseOptions(controller: self, availableProducts: products, accountNumber: interactor.accountNumber)
                } else {
                    delegate?.didRequestToShowFailToFetchProducts(controller: self)
                }
            case .failure:
                delegate?.didRequestToShowFailToFetchProducts(controller: self)
            }
            contentView.isFetchingProducts = false
        }
    }

    func didTapRedeemVoucherButton(welcomeContentView: WelcomeContentView, button: AppButton) {
        delegate?.didRequestToRedeemVoucher(controller: self)
    }
}

extension WelcomeViewController: @preconcurrency InAppPurchaseViewControllerDelegate {
    func didBeginPayment() {
        contentView.isPurchasing = true
    }

    func didEndPayment() {
        contentView.isPurchasing = false
    }
}
