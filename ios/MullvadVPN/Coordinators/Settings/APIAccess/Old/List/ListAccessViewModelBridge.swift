import SwiftUI

class ListAccessViewModelBridge: ListAccessViewModel {
    private let interactor: ListAccessMethodInteractorProtocol
    private weak var delegate: ListAccessMethodViewControllerDelegate?

    @Published var items: [ListAccessMethodItem] = []
    @Published var itemInUse: ListAccessMethodItem?

    init(
        interactor: ListAccessMethodInteractorProtocol,
        delegate: ListAccessMethodViewControllerDelegate?
    ) {
        self.interactor = interactor
        self.delegate = delegate

        interactor.itemsPublisher.assign(to: &$items)
        interactor.itemInUsePublisher.assign(to: &$itemInUse)
    }

    func cipherIsValid(for item: ListAccessMethodItem) -> Bool {
        let ciphers = interactor.shadowsocksCiphers
        let method = interactor.accessMethod(by: item.id)

        return if case .shadowsocks(let config) = method?.proxyConfiguration {
            ciphers.contains(config.cipher)
        } else {
            true
        }
    }

    func addNewMethod() {
        delegate?.controllerShouldAddNew()
    }

    func methodSelected(_ method: ListAccessMethodItem) {
        delegate?.controller(shouldEditItem: method)
    }

    func showAbout() {
        delegate?.controllerShouldShowAbout()
    }
}
