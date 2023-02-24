use std::{ops::Range, sync::Arc};

use super::app::AppActions;
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode, KeyEvent};
use mullvad_management_interface::{
    types::{self, Relay as RelayListRelay, RelayListCity, RelayListCountry},
    ManagementServiceClient,
};
use parking_lot::Mutex;
use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{List, ListItem, Paragraph},
};

#[derive(Clone)]
pub struct LocationCode(String, Option<String>, Option<String>);

#[derive(Clone)]
struct Location {
    name: String,
    location: LocationCode,
    expanded: bool,
    indentation: usize,
    subtree: Vec<Location>,
}

impl Location {
    fn can_expand(&self) -> bool {
        !self.subtree.is_empty()
    }

    fn toggle_expanded(&mut self) {
        if self.can_expand() {
            self.expanded = !self.expanded;
        }
    }
}

impl ToString for Location {
    fn to_string(&self) -> String {
        let name = if self.subtree.is_empty() {
            self.name.clone()
        } else {
            if self.expanded {
                format!("- {}", self.name)
            } else {
                format!("+ {}", self.name)
            }
        };

        format!(
            "{:>indentation$} {}",
            "",
            name,
            indentation = self.indentation * 2
        )
    }
}

struct SelectLocation {
    actions: AppActions,
    locations: Vec<Location>,
    index: usize,
    rpc: ManagementServiceClient,
}

impl SelectLocation {
    pub fn new(
        actions: AppActions,
        rpc: ManagementServiceClient,
        locations: Vec<Location>,
    ) -> Self {
        Self {
            actions,
            locations,
            index: 0,
            rpc,
        }
    }

    fn flat_locations(&self) -> Vec<Location> {
        let mut location_list: Vec<Location> = Vec::new();

        for country in &self.locations {
            location_list.push(country.clone());
            if country.expanded {
                for city in &country.subtree {
                    location_list.push(city.clone());
                    if city.expanded {
                        for relay in &city.subtree {
                            location_list.push(relay.clone());
                        }
                    }
                }
            }
        }

        location_list
    }

    fn mut_get_current(&mut self) -> &mut Location {
        let mut items_left = self.index;

        for country in self.locations.iter_mut() {
            if items_left == 0 {
                return country;
            }

            items_left -= 1;
            if country.expanded {
                for city in country.subtree.iter_mut() {
                    if items_left == 0 {
                        return city;
                    }

                    items_left -= 1;
                    if city.expanded {
                        for relay in city.subtree.iter_mut() {
                            if items_left == 0 {
                                return relay;
                            }

                            items_left -= 1;
                        }
                    }
                }
            }
        }

        panic!();
    }

    fn list_widget(&self, height: usize) -> List<'_> {
        let locations = self.flat_locations();
        let range = Self::list_range(height / 2 + 2, locations.len(), self.index);
        let items: Vec<ListItem<'_>> = locations
            .iter()
            .cloned()
            .enumerate()
            .skip(range.start)
            .take(range.len())
            .flat_map(|(i, item)| {
                let mut items: Vec<ListItem<'_>> = Vec::new();
                if i == self.index {
                    items.push(
                        ListItem::new(item.to_string()).style(Style::default().fg(Color::Green)),
                    );
                } else {
                    items.push(ListItem::new(item.to_string()));
                };

                items.push(ListItem::new("------------------------------------------"));

                items
            })
            .collect();

        List::new(items)
    }

    fn list_range(height: usize, items_length: usize, current_index: usize) -> Range<usize> {
        if height >= items_length || current_index < height / 2 {
            0..height
        } else if current_index >= items_length - height / 2 {
            let start_index = items_length - height + 1;
            let end_index = start_index + height;
            start_index..end_index
        } else {
            let start_index = current_index - (height / 2);
            let end_index = start_index + height;
            start_index..end_index
        }
    }

    fn move_up(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    fn move_down(&mut self) {
        let max = self.flat_locations().len() - 1;
        self.index = self.index.saturating_add(1).clamp(0, max);
    }

    fn move_top(&mut self) {
        self.index = 0;
    }

    fn move_bottom(&mut self) {
        self.index = self.flat_locations().len() - 1;
    }

    fn toggle_expanded(&mut self) {
        self.mut_get_current().toggle_expanded();
    }

    fn select(&self) {
        let location = self
            .flat_locations()
            .get(self.index)
            .unwrap()
            .location
            .clone();

        let relay_location = match location {
            LocationCode(country, None, None) => types::RelayLocation {
                country,
                ..Default::default()
            },
            LocationCode(country, Some(city), None) => types::RelayLocation {
                country,
                city,
                ..Default::default()
            },
            LocationCode(country, Some(city), Some(hostname)) => types::RelayLocation {
                country,
                city,
                hostname,
            },
            _ => panic!(),
        };

        let mut rpc = self.rpc.clone();
        tokio::spawn(async move {
            let update = types::RelaySettingsUpdate {
                r#type: Some(types::relay_settings_update::Type::Normal(
                    types::NormalRelaySettingsUpdate {
                        location: Some(relay_location),
                        ..Default::default()
                    },
                )),
            };

            rpc.update_relay_settings(update).await.unwrap();
        });
    }
}

impl Component for SelectLocation {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let close_area = Rect::new(area.x + 1, area.y, 7, 1);
        let title_area = Rect::new(area.x + area.width / 2 - 7, area.y, area.width, 1);
        let list_area = Rect::new(area.x + 1, area.y + 2, area.width - 2, area.height - 2);

        f.render_widget(Paragraph::new("X (esc)"), close_area);
        f.render_widget(Paragraph::new("Select location"), title_area);
        f.render_widget(self.list_widget(list_area.height.into()), list_area);
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up => self.move_up(),
                KeyCode::Char('k') => self.move_up(),
                KeyCode::Down => self.move_down(),
                KeyCode::Char('j') => self.move_down(),
                KeyCode::PageUp => self.move_top(),
                KeyCode::Char('g') => self.move_top(),
                KeyCode::PageDown => self.move_bottom(),
                KeyCode::Char('G') => self.move_bottom(),
                KeyCode::Char(' ') => self.toggle_expanded(),
                KeyCode::Char('+') => self.toggle_expanded(),
                KeyCode::Char('-') => self.toggle_expanded(),
                KeyCode::Enter => self.select(),
                _ => return,
            }
            self.actions.redraw();
        }
    }
}

pub struct SelectLocationContainer {
    child: Arc<Mutex<Option<SelectLocation>>>,
}

impl SelectLocationContainer {
    pub fn new(actions: AppActions, mut rpc: ManagementServiceClient) -> Self {
        let child = Arc::new(Mutex::new(None));

        let async_child = child.clone();
        tokio::spawn(async move {
            let countries = rpc
                .get_relay_locations(())
                .await
                .unwrap()
                .into_inner()
                .countries;

            let locations: Vec<Location> =
                countries.into_iter().map(Self::convert_country).collect();
            {
                let mut child = async_child.lock();
                *child = Some(SelectLocation::new(actions.clone(), rpc, locations));
            }

            actions.redraw_async().await;
        });

        Self { child }
    }

    fn convert_country(country: RelayListCountry) -> Location {
        Location {
            name: country.name,
            location: LocationCode(country.code.clone(), None, None),
            expanded: false,
            indentation: 0,
            subtree: country
                .cities
                .into_iter()
                .map(|city| Self::convert_city(city, country.code.clone()))
                .collect(),
        }
    }

    fn convert_city(city: RelayListCity, country_code: String) -> Location {
        Location {
            name: city.name,
            location: LocationCode(country_code.clone(), Some(city.code.clone()), None),
            expanded: false,
            indentation: 1,
            subtree: city
                .relays
                .into_iter()
                .map(|relay| Self::convert_relay(relay, country_code.clone(), city.code.clone()))
                .collect(),
        }
    }

    fn convert_relay(relay: RelayListRelay, country_code: String, city_code: String) -> Location {
        Location {
            name: relay.hostname.clone(),
            location: LocationCode(country_code, Some(city_code), Some(relay.hostname)),
            expanded: false,
            indentation: 2,
            subtree: Vec::new(),
        }
    }
}

impl Component for SelectLocationContainer {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        if let Some(ref mut child) = *self.child.lock() {
            child.draw(f, area);
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(ref mut child) = *self.child.lock() {
            child.handle_event(event);
        }
    }
}
