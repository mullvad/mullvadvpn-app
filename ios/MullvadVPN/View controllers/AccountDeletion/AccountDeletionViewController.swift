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

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        disableEditing()
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        contentView.isEditing = false
        super.viewWillTransition(to: size, with: coordinator)
    }

    private func enableEditing() {
        guard !contentView.isEditing else { return }
        contentView.isEditing = true
    }

    private func disableEditing() {
        guard contentView.isEditing else { return }
        contentView.isEditing = false
    }

    private func configureUI() {
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all())
        }
    }

    private func submit(accountNumber: String) {
        contentView.state = .loading
        task = interactor.delete(accountNumber: accountNumber) { [weak self] error in
            guard let self else { return }
            guard let error else {
                self.contentView.state = .initial
                self.delegate?.deleteAccountDidSucceed(controller: self)
                return
            }
            self.contentView.state = .failure(error)
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
            submit(accountNumber: accountNumber)
        case let .failure(error):
            contentView.state = .failure(error)
        }
    }
}
