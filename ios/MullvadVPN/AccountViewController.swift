//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class AccountViewController: UIViewController {

    @IBOutlet var accountLabel: UILabel!
    @IBOutlet var expiryLabel: UILabel!

    private var accountExpiryObserver: AccountExpiryRefresh.Observer?

    override func viewDidLoad() {
        super.viewDidLoad()

        updateView()
        startAccountExpiryUpdates()
    }

    // MARK: - Actions

    @IBAction func doBuyCredits() {
        UIApplication.shared.open(WebLinks.purchaseURL, options: [:])
    }

    @IBAction func doLogout() {
        Account.logout()

        performSegue(withIdentifier: SegueIdentifier.Account.logout.rawValue, sender: self)
    }

    // MARK: - Private

    private func updateView() {
        accountLabel.text = Account.token

        if let expiryDate = Account.expiry {
            let accountExpiry = AccountExpiry(date: expiryDate)

            if accountExpiry.isExpired {
                expiryLabel.text = NSLocalizedString("OUT OF TIME", tableName: "Settings", comment: "")
                expiryLabel.textColor = .dangerColor
            } else {
                expiryLabel.text = accountExpiry.formattedDate
                expiryLabel.textColor = .white
            }
        }
    }

    private func startAccountExpiryUpdates() {
        accountExpiryObserver = AccountExpiryRefresh.shared
            .startMonitoringUpdates(with: { [weak self] (expiryDate) in
                self?.updateView()
            })
    }
}
