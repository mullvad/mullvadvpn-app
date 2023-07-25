//
//  AccountDeletionViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

protocol AccountDeletionViewControllerDelegate: AnyObject {
    func deleteAccountDidSucceed(controller: AccountDeletionViewController)
    func deleteAccountDidCancel(controller: AccountDeletionViewController)
}

class AccountDeletionViewController: UIViewController {
    private var task: Cancellable?
    private lazy var contentView: AccountDeletionContentView = {
        let view = AccountDeletionContentView()
        view.delegate = self
        view.isEditing = true
        return view
    }()

    weak var delegate: AccountDeletionViewControllerDelegate?
    var interactor: AccountDeletionInteractor

    init(interactor: AccountDeletionInteractor) {
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

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        enableEditing()
    }

    private func configureUI() {
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all())
        }
    }

    private func enableEditing() {
        guard !contentView.isEditing else { return }
        contentView.isEditing = true
    }

    private func submit(accountNumber: String) {
        contentView.state = .loading
        task = interactor.delete(accountNumber: accountNumber) { [weak self] result in
            guard let self else { return }
            switch result {
            case .success:
                self.contentView.state = .initial
                self.delegate?.deleteAccountDidSucceed(controller: self)
            case let .failure(error):
                self.contentView.state = .failure(error)
            }
        }
    }
}

extension AccountDeletionViewController: AccountDeletionContentViewDelegate {
    func didTapCancelButton(contentView: AccountDeletionContentView, button: AppButton) {
        contentView.isEditing = false
        task?.cancel()
        delegate?.deleteAccountDidCancel(controller: self)
    }

    func didTapDeleteButton(contentView: AccountDeletionContentView, button: AppButton) {
        switch interactor.validate(input: contentView.lastPartOfAccountNumber) {
        case let .success(accountNumber):
            self.submit(accountNumber: accountNumber)
        case let .failure(error):
            self.contentView.state = .failure(error)
        }
    }
}
