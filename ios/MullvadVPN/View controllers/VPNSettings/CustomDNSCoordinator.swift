import Routing
import UIKit

class CustomDNSCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let interactor: VPNSettingsInteractor
    private let route: AppRoute

    var didFinish: ((CustomDNSCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(navigationController: UINavigationController, interactor: VPNSettingsInteractor, route: AppRoute) {
        self.interactor = interactor
        self.navigationController = navigationController
        self.route = route
    }

    func start(animated: Bool) {
        let alertPresenter = AlertPresenter(context: self)
        let viewController = CustomDNSViewController(interactor: interactor, alertPresenter: alertPresenter)
        customiseNavigation(on: viewController)
        navigationController.pushViewController(viewController, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .dnsSettings {
            navigationController.navigationItem.largeTitleDisplayMode = .always
            navigationController.navigationBar.prefersLargeTitles = true
            let doneButton = UIBarButtonItem(
                systemItem: .done,
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                })
            )
            viewController.navigationItem.rightBarButtonItem = doneButton
        }
    }
}
