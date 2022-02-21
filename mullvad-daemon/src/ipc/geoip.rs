use super::oneshot_send;
use crate::{Daemon, EventListener};
use futures::{channel::oneshot, future::Either};
use mullvad_types::{location::GeoIpLocation, states::TunnelState};

/// Returns the current location when the tunnel is either disconnected or connected. In any other
/// tunnel state, `None` is returned.
impl<L: EventListener + Clone + Send + 'static> Daemon<L> {
    pub(super) fn on_get_current_location(&mut self, tx: oneshot::Sender<Option<GeoIpLocation>>) {
        use TunnelState::*;

        let relay_location = match &self.tunnel_state {
            Connecting { location, .. } | Connected { location, .. } => location.clone(),
            Disconnecting(..) => self.build_location_from_relay(),
            _ => None,
        };

        let location_fut = match &self.tunnel_state {
            Connected { .. } | Disconnected => {
                let location_fut = self.get_geo_location();
                Either::Left(async move {
                    location_fut
                        .await
                        .ok()
                        .map(|fetched_location| GeoIpLocation {
                            ipv4: fetched_location.ipv4,
                            ipv6: fetched_location.ipv6,
                            ..relay_location.unwrap_or(fetched_location)
                        })
                })
            }
            _ => Either::Right(async move { relay_location }),
        };

        tokio::spawn(async move {
            oneshot_send(tx, location_fut.await, "current location");
        });
    }
}
