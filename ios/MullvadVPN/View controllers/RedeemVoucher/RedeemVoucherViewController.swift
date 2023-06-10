//
//  RedeemVoucherViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import UIKit

protocol RedeemVoucherViewControllerDelegate: AnyObject {
    func redeemVoucherDidSuccess(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    )
    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController)
}

class RedeemVoucherViewController: UIViewController, UINavigationControllerDelegate {
    private let contentView = RedeemVoucherContentView()
    private var isBecameFirstResponder = false

    weak var delegate: RedeemVoucherViewControllerDelegate?

    init() {
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    // MARK: - Life Cycle

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        addActions()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        becameFirstResponder()
    }

    // MARK: - private functions

    private func becameFirstResponder() {
        guard !isBecameFirstResponder else { return }
        isBecameFirstResponder = true
        contentView.isEditing = true
    }

    private func addActions() {
        contentView.redeemAction = { [weak self] code in
            self?.submit(code: code)
        }

        contentView.cancelAction = { [weak self] in
            self?.cancel()
        }
    }

    private func configureUI() {
        view.addSubview(contentView)
        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func submit(code: String) {
        let day  = 24 * 3600
        delegate?.redeemVoucherDidSuccess(self, with: REST.SubmitVoucherResponse(
            timeAdded: day,
            newExpiry: Date()
                .addingTimeInterval(TimeInterval(day))
        ))
    }

    private func cancel() {
        delegate?.redeemVoucherDidCancel(self)
    }
}
