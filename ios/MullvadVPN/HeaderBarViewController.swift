//
//  HeaderBarViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

protocol HeaderBarViewControllerDelegate: class {
    func headerBarViewControllerShouldOpenSettings(_ controller: HeaderBarViewController)
}

class HeaderBarViewController: UIViewController {
    weak var delegate: HeaderBarViewControllerDelegate?
    
    @IBAction func handleSettingsButton() {
        self.delegate?.headerBarViewControllerShouldOpenSettings(self)
    }
}
