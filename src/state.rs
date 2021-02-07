use crate::{
    datetime_from_iso,
    error::Error,
    Event,
    Interval,
};
use tui::widgets::{ ListState, TableState };

pub enum Focus {
    InputAdd,
    Intervals,
    Timed,
    Untimed,
    None,
}

pub struct State {
    pub buffer: String,
    pub focus: Focus,
    pub intervals: Vec<Event>,
    pub intervals_offset: usize,
    pub intervals_state: TableState,
    pub last_error: Option<Error>,
    pub timed: Vec<Event>,
    pub timed_offset: usize,
    pub timed_state: TableState,
    pub untimed: Vec<Event>,
    pub untimed_offset: usize,
    pub untimed_state: ListState,
}

impl State {
    pub fn add_event_from_buffer(&mut self) -> Result<(), Error> {
        if self.buffer.is_empty() { return Err(Error::NoInfo); }
        if !self.buffer.contains('\t') { return Err(Error::InvalidRecord); }
        let mut event = Event {
            start: None,
            interval: Interval::None,
            description: String::new(),
        };
        let mut halves = self.buffer.split('\t');
        match halves.next() {
            Some("") => {}, // leave event.start empty.
            Some(iso) => {
                let mut tokens = iso.split('/');
                match (tokens.next(), tokens.next(), tokens.next()) {
                    (Some(repetition), Some(start), Some(end)) => {
                        match repetition.strip_prefix('R') {
                            Some("") => event.interval = Interval::RepIndefinite(datetime_from_iso(end)?),
                            Some(string) => {
                                let occurrences = match string.parse::<usize>() {
                                    Ok(num) => num,
                                    Err(_) => return Err(Error::InvalidIso),
                                };
                                event.interval = Interval::RepDefinite {
                                    occurrences,
                                    end: datetime_from_iso(end)?,
                                };
                            },
                            None => return Err(Error::InvalidIso),
                        }
                        event.start = Some(datetime_from_iso(start)?);
                    },
                    (Some(start), Some(end), None) => {
                        event.start = Some(datetime_from_iso(start)?);
                        event.interval = Interval::Standard(datetime_from_iso(end)?);
                    },
                    (Some(start), None, None) => {
                        event.start = Some(datetime_from_iso(start)?);
                        event.interval = Interval::None;
                    },
                    _ => {},
                }
            },
            None => {}, // leave event.start empty
        }
        match halves.next() {
            Some(string) => event.description.push_str(string.trim()),
            None => {}, // leave event.description empty
        }
        if event.start == None && event.description == "" { return Err(Error::NoInfo); }
        match (event.start, &event.interval) {
            (None, _) => self.untimed.push(event),
            (Some(_), Interval::Standard(_)) => {
                self.intervals.push(event);
                self.intervals.sort_unstable();
            },
            (Some(_), _) => {
                self.timed.push(event);
                self.timed.sort_unstable();
            },
        }
        self.buffer.clear();
        Ok(())
    }

    pub fn delete_selected(&mut self) {
        match self.last_error {
            Some(Error::DeletionWarning) => {
                match self.focus {
                    Focus::Intervals => { self.intervals.remove(self.intervals_state.selected().unwrap()); },
                    Focus::Timed => { self.timed.remove(self.timed_state.selected().unwrap()); },
                    Focus::Untimed => { self.untimed.remove(self.untimed_state.selected().unwrap()); },
                    _ => {}, // deletion can't happen anywhere else.
                }
                self.last_error = None;
            },
            _ => match self.focus {
                Focus::Intervals
                    | Focus::Timed
                    | Focus::Untimed
                => self.last_error = Some(Error::DeletionWarning),
                _ => {},
            },
        }
    }

    pub fn focus(&mut self, target: Focus) -> Result<(), Error> {
        match self.focus {
            Focus::Intervals => self.intervals_state.select(None),
            Focus::Timed => self.timed_state.select(None),
            Focus::Untimed => self.untimed_state.select(None),
            _ => {},
        }
        match target {
            Focus::Intervals => if !self.intervals.is_empty() { self.intervals_state.select(Some(self.intervals_offset)) },
            Focus::Timed => if !self.timed.is_empty() { self.timed_state.select(Some(self.timed_offset)) },
            Focus::Untimed => if !self.untimed.is_empty() { self.untimed_state.select(Some(self.untimed_offset)) },
            _ => {},
        }
        self.focus = target;
        Ok(())
    }

    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::Intervals => match self.intervals_state.selected() {
                Some(selected) if selected < self.intervals.len() - 1 => {
                    self.intervals_offset = selected + 1;
                    self.intervals_state.select(Some(self.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match self.timed_state.selected() {
                Some(selected) if selected < self.timed.len() - 1 => {
                    self.timed_offset = selected + 1;
                    self.timed_state.select(Some(self.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match self.untimed_state.selected() {
                Some(selected) if selected < self.untimed.len() - 1 => {
                    self.untimed_offset = selected + 1;
                    self.untimed_state.select(Some(self.untimed_offset));
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::Intervals => match self.intervals_state.selected() {
                Some(selected) if selected > 0 => {
                    self.intervals_offset = selected - 1;
                    self.intervals_state.select(Some(self.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match self.timed_state.selected() {
                Some(selected) if selected > 0 => {
                    self.timed_offset = selected - 1;
                    self.timed_state.select(Some(self.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match self.untimed_state.selected() {
                Some(selected) if selected > 0 => {
                    self.untimed_offset = selected - 1;
                    self.untimed_state.select(Some(self.untimed_offset));
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn take_all(&mut self) -> Vec<Event> {
        let mut vec: Vec<Event> = Vec::new();
        vec.append(&mut self.intervals);
        vec.append(&mut self.timed);
        vec.append(&mut self.untimed);
        vec
    }
}
