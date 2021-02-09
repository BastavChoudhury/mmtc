use clap::{AppSettings, Clap};
use tui::widgets::ListState;

use std::{net::SocketAddr, path::PathBuf};

use crate::mpd::{Status, Track};

/// Minimal mpd terminal client that aims to be simple yet highly configurable
///
/// Homepage: https://github.com/figsoda/mmtc
#[derive(Clap)]
#[clap(version, global_setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// Clear query on play
    #[clap(long, multiple_occurrences(true))]
    pub clear_query_on_play: bool,

    /// Cycle through the queue
    #[clap(long, multiple_occurrences(true))]
    pub cycle: bool,

    /// Don't clear query on play
    #[clap(
        long,
        multiple_occurrences(true),
        overrides_with("clear-query-on-play")
    )]
    pub no_clear_query_on_play: bool,

    /// Don't cycle through the queue
    #[clap(long, multiple_occurrences(true), overrides_with("cycle"))]
    pub no_cycle: bool,

    /// Specify the address of the mpd server
    #[clap(long, value_name = "address")]
    pub address: Option<SocketAddr>,

    /// Specify the config file
    #[clap(short, long, value_name = "file")]
    pub config: Option<PathBuf>,

    /// The number of lines to jump
    #[clap(long, value_name = "number")]
    pub jump_lines: Option<usize>,

    /// The time to seek in seconds
    #[clap(long, value_name = "number")]
    pub seek_secs: Option<f32>,

    /// The amount of status updates per second
    #[clap(long, value_name = "number")]
    pub ups: Option<f32>,
}

pub struct State {
    pub status: Status,
    pub queue: Vec<Track>,
    pub selected: usize,
    pub liststate: ListState,
    pub searching: bool,
    pub query: String,
    pub filtered: Vec<usize>,
}

#[derive(Debug)]
pub enum Command {
    Quit,
    ToggleRepeat,
    ToggleRandom,
    ToggleSingle,
    ToggleOneshot,
    ToggleConsume,
    TogglePause,
    Stop,
    SeekBackwards,
    SeekForwards,
    Previous,
    Next,
    Play,
    Reselect,
    Down,
    Up,
    JumpDown,
    JumpUp,
    GotoTop,
    GotoBottom,
    InputSearch(char),
    BackspaceSearch,
    QuitSearch,
    Searching(bool),
}

impl State {
    pub fn select(&mut self, x: usize) {
        self.selected = x;
        self.liststate.select(Some(x));
    }

    pub fn reselect(&mut self) {
        self.select(self.status.song.as_ref().map_or(0, |song| song.pos));
    }

    pub fn len(&self) -> usize {
        if self.query.is_empty() {
            self.queue.len()
        } else {
            self.filtered.len()
        }
    }

    pub fn update_search(&mut self, queue_strings: &[String]) {
        let query = self.query.to_lowercase();
        self.filtered.clear();
        for (i, track) in queue_strings.iter().enumerate() {
            if track.contains(&query) {
                self.filtered.push(i);
            }
        }
        self.liststate.select(None);
        self.select(0);
    }

    pub fn quit_search(&mut self) {
        self.searching = false;
        if !self.query.is_empty() {
            self.query.clear();
            self.reselect();
        }
    }
}
