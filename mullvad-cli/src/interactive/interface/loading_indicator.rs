use std::{sync::Arc, time::Duration};

use super::app::AppActions;
use crate::interactive::component::{Component, Frame};

use futures_timer::Delay;
use parking_lot::Mutex;
use tui::{
    layout::{Alignment, Rect},
    widgets::Paragraph,
};

#[derive(Clone)]
pub struct LoadingIndicator {
    phase: Arc<Mutex<U3>>,
}

impl LoadingIndicator {
    pub fn new(actions: AppActions) -> Self {
        let phase = Arc::new(Mutex::new(U3::default()));
        Self::cycle(actions, phase.clone());
        Self { phase }
    }

    fn cycle(actions: AppActions, phase: Arc<Mutex<U3>>) {
        tokio::spawn(async move {
            loop {
                Delay::new(Duration::from_millis(100)).await;
                phase.lock().increment();
                actions.redraw_async().await;
            }
        });
    }

    fn push_str_if(string: &mut String, to_push: &str, condition: bool) {
        if condition {
            string.push_str(&to_push);
        } else {
            let to_push = format!("{:width$}", " ", width = to_push.len());
            string.push_str(&to_push);
        }
    }
}

impl Component for LoadingIndicator {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let phase = self.phase.lock().value();

        let mut text = String::new();
        Self::push_str_if(&mut text, "__", phase < 3);
        text.push_str("\n");
        Self::push_str_if(&mut text, "/", phase > 4 || phase < 2);
        text.push_str("  ");
        Self::push_str_if(&mut text, "\\", phase > 1 && phase < 4);
        text.push_str("\n");
        Self::push_str_if(&mut text, "\\", phase > 3 || phase < 1);
        Self::push_str_if(&mut text, "__", phase > 3);
        Self::push_str_if(&mut text, "/", phase > 2 && phase < 5);

        let value = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(value, area);
    }
}

#[derive(Default)]
struct U3(u8);

impl U3 {
    pub fn increment(&mut self) {
        if self.0 == 5 {
            self.0 = 0;
        } else {
            self.0 += 1;
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}
