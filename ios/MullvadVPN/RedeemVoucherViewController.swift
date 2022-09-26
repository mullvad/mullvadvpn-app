//
//  RedeemVoucherViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

protocol RedeemVoucherResponseProtocol: AnyObject {
    func redeemedVoucherSuccessfully()
    func redeemedVoucherWithError(error: String)
}

extension RedeemVoucherResponseProtocol {
    func redeemedVoucherWithError(error: String) {}
}

class RedeemVoucherViewController: UIViewController {
    // MARK: - Constants

    private let apiProxy: REST.APIProxy

    // MARK: - Views

    private lazy var contentView = RedeemVoucherContentView { [weak self] in
        self?.submitVoucher()
    } cancelAction: { [weak self] in
        self?.dismiss(animated: true)
    }

    // MARK: - Variables

    public weak var delegate: RedeemVoucherResponseProtocol?

    private var redeemVoucherState = RedeemVoucherState.initial
    private var navigationControllerOriginY: CGFloat?
    private var isViewMoved = false

    private var isVoucherLengthSatisfied = false {
        didSet {
            if isVoucherLengthSatisfied != oldValue {
                updateViews(with: self.redeemVoucherState, animated: true)
            }
        }
    }

    // MARK: - Lifecycles

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }

    init(apiProxy: REST.APIProxy = REST.ProxyFactory.shared.createAPIProxy()) {
        self.apiProxy = apiProxy

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        setUpContentView()
        addObservers()
        updateViews(with: .initial, animated: false)

        navigationController?.delegate = self
    }

    // MARK: - View setup

    private func setUpContentView() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }
}

// MARK: - Private Functions

private extension RedeemVoucherViewController {
    private func addObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: contentView.inputTextField
        )
    }

    @objc private func textDidChange() {
        updateIsVoucherLengthSatisfied()
        if isVoucherLengthSatisfied {
            dismissKeyboard()
        }
    }

    private func updateIsVoucherLengthSatisfied() {
        isVoucherLengthSatisfied =
            contentView.inputTextField.text?.count == contentView.inputTextField.placeholder?.count
    }

    private func dismissKeyboard() {
        contentView.inputTextField.resignFirstResponder()
    }

    private func setRedeemVoucherState(_ state: RedeemVoucherState, animated: Bool) {
        redeemVoucherState = state
        updateViews(with: state, animated: true)
    }

    private func updateViews(with state: RedeemVoucherState, animated: Bool) {
        switch state {
        case let .success(timeAdded):
            delegate?
                .redeemedVoucherSuccessfully()

            navigationController?
                .pushViewController(
                    RedeemVoucherSucceededViewController(timeAdded: timeAdded),
                    animated: true
                )
        default:
            if animated {
                UIView.animate(
                    withDuration: 0.8,
                    delay: 0,
                    usingSpringWithDamping: 0.5,
                    initialSpringVelocity: 6.9,
                    options: .curveEaseInOut,
                    animations: {
                        self.updateViewsAccordingToState(with: state)
                    }
                )
            } else {
                updateViewsAccordingToState(with: state)
            }
        }
    }

    private func updateViewsAccordingToState(with state: RedeemVoucherState) {
        contentView
            .updateViews(
                state: state,
                isVoucherLengthSatisfied: isVoucherLengthSatisfied
            )
    }

    private func submitVoucher() {
        guard let voucherCode = contentView.inputTextField.text,
              let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        setRedeemVoucherState(.waiting, animated: true)

        let request = REST.SubmitVoucherRequest(voucherCode: voucherCode)

        // AIM: - Keeping animations run smoothly
        // Adding delay based on speed of api response
        DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
            self.apiProxy.submitVoucher(
                request,
                accountNumber: accountNumber,
                retryStrategy: .default
            ) { completion in
                switch completion {
                case let .success(submitVoucherResponse):
                    self.setRedeemVoucherState(.success(
                        self.formattedTimeAdded(from: submitVoucherResponse.timeAdded)
                    ), animated: true)
                case .failure:
                    self.setRedeemVoucherState(.failure, animated: true)
                default:
                    break
                }
            }
        }
    }

    private func formattedTimeAdded(from timeAdded: Int) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full

        return formatter.string(from: Double(timeAdded)) ?? ""
    }
}

// MARK: - Redeem Voucher State

extension RedeemVoucherViewController {
    enum RedeemVoucherState: Equatable {
        case success(String)
        case initial, waiting, failure

        var isWaiting: Bool {
            self == .waiting
        }

        func getStatusLabelText(timeAdded: String = "") -> String {
            switch self {
            case .success:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_SUCCESS",
                    tableName: "RedeemVoucher",
                    value: "\(timeAdded) were added to your account.",
                    comment: ""
                )
            case .failure:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_FAILURE",
                    tableName: "RedeemVoucher",
                    value: "Voucher code is invalid.",
                    comment: ""
                )
            default:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_WAITING",
                    tableName: "RedeemVoucher",
                    value: "Verifying voucher...",
                    comment: ""
                )
            }
        }
    }
}

extension RedeemVoucherViewController: UINavigationControllerDelegate {
    func navigationController(
        _ navigationController: UINavigationController,
        animationControllerFor operation: UINavigationController.Operation,
        from fromVC: UIViewController,
        to toVC: UIViewController
    ) -> UIViewControllerAnimatedTransitioning? {
        // INFO: use UINavigationControllerOperation.push or UINavigationControllerOperation.pop to detect the 'direction' of the navigation

        class FadeAnimation: NSObject, UIViewControllerAnimatedTransitioning {
            func transitionDuration(using transitionContext: UIViewControllerContextTransitioning?)
                -> TimeInterval
            {
                return 0.3
            }

            func animateTransition(using transitionContext: UIViewControllerContextTransitioning) {
                let toViewController = transitionContext
                    .viewController(forKey: UITransitionContextViewControllerKey.to)
                if let vc = toViewController {
                    transitionContext.finalFrame(for: vc)
                    transitionContext.containerView.addSubview(vc.view)
                    vc.view.alpha = 0.0
                    UIView.animate(
                        withDuration: transitionDuration(using: transitionContext),
                        animations: {
                            vc.view.alpha = 1.0
                        },
                        completion: { finished in
                            transitionContext
                                .completeTransition(!transitionContext.transitionWasCancelled)
                        }
                    )
                } else {
                    preconditionFailure("Oops! Something went wrong! 'ToView' controller is nil")
                }
            }
        }

        return FadeAnimation()
    }
}
