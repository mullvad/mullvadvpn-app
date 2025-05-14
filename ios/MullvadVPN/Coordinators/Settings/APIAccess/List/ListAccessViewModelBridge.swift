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
