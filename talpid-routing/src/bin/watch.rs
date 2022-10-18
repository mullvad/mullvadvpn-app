use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
};

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
    println!("order_of-interfaces {}", order_of_interfaces().join(", "));
    rt.block_on(talpid_routing::watch_routes())
        .expect("rt panicked");
}

fn order_of_interfaces() -> Vec<String> {
    let prefs = SCPreferences::default(&CFString::new("talpid-routing"));
    let services = SCNetworkService::get_services(&prefs);
    let set = SCNetworkSet::new(&prefs);
    let service_order = set.service_order();

    service_order
        .iter()
        .filter_map(|service_id| {
            services
                .iter()
                .find(|service| service.id().as_ref() == Some(&*service_id))
                .and_then(|service| service.network_interface()?.bsd_name())
                .map(|cf_name| cf_name.to_string())
        })
        .collect::<Vec<_>>()
}
