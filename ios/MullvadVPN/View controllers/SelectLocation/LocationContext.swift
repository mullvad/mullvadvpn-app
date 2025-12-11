struct LocationContext {
    var locations: [LocationNode]
    var allLocations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    let selectLocation: (LocationNode) -> Void

    init(
        locations: [LocationNode] = [],
        allLocations: [LocationNode] = [],
        customLists: [LocationNode] = [],
        filter: [SelectLocationFilter] = [],
        selectedLocation: LocationNode? = nil,
        connectedRelayHostname: String? = nil,
        selectLocation: @escaping (LocationNode) -> Void = { _ in }
    ) {
        self.locations = locations
        self.allLocations = allLocations
        self.customLists = customLists
        self.filter = filter
        self.selectLocation = selectLocation
    }

    var selectedLocation: LocationNode? {
        (locations + customLists)
            .flatMap { [$0] + $0.flattened }
            .first { $0.isSelected }
    }
}

extension LocationContext {
    var visibleLocations: [LocationNode] {
        locations.filter { !$0.isHiddenFromSearch }
    }
}
