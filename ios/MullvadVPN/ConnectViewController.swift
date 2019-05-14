//
//  ConnectViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class ConnectViewController: UIViewController, HeaderBarViewControllerDelegate {

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func prepare(for segue: UIStoryboardSegue, sender: Any?) {
        if case .embedHeader? = SegueIdentifier.Connect.from(segue: segue) {
            let headerBarController = segue.destination as? HeaderBarViewController
            headerBarController?.delegate = self
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        // Do any additional setup after loading the view, typically from a nib.
    }

    // MARK: - HeaderBarViewControllerDelegate

    func headerBarViewControllerShouldOpenSettings(_ controller: HeaderBarViewController) {
        performSegue(withIdentifier: SegueIdentifier.Connect.showSettings.rawValue, sender: self)
    }

    // MARK: - Actions

    @IBAction func unwindFromSelectLocation(segue: UIStoryboardSegue) {

    }

}
