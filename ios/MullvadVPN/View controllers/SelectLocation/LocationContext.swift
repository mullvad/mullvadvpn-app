struct LocationContext {
    var recents: [LocationNode]
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    let selectLocation: (LocationNode) -> Void

    init(
        recents: [LocationNode] = [],
        locations: [LocationNode] = [],
        customLists: [LocationNode] = [],
        filter: [SelectLocationFilter] = [],
        selectedLocation: LocationNode? = nil,
        connectedRelayHostname: String? = nil,
        selectLocation: @escaping (LocationNode) -> Void = { _ in }
    ) {
        self.recents = recents
        self.locations = locations
        self.customLists = customLists
        self.filter = filter
        self.selectLocation = selectLocation
    }

    var selectedLocation: LocationNode? {
        (recents + customLists + locations)
            .flatMap { $0.flattened + [$0] }
            .first { $0.isSelected }
    }
}
