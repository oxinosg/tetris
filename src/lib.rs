#![recursion_limit = "2048"]

#[macro_use]
extern crate lazy_static;
extern crate stdweb;

use log::info;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use stdweb::traits::*;
use stdweb::web::document;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, ToString};
use yew::events::IKeyboardEvent;
use yew::format::Json;
use yew::services::storage::{Area, StorageService};
use yew::services::{IntervalService, Task};
use yew::{html, Callback, Component, ComponentLink, Href, Html, KeyDownEvent, ShouldRender};

lazy_static! {
    #[derive(Debug)]
    static ref PIECES: HashMap<&'static str, Piece> = {
        let mut map = HashMap::new();
        map.insert(
            PieceType::E.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 1,
                    n_cols: 1,
                    data: vec![PieceType::E]
                }
            },
        );
        map.insert(
            PieceType::I.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 4,
                    n_cols: 4,
                    data: vec![
                        PieceType::E, PieceType::I, PieceType::E, PieceType::E,
                        PieceType::E, PieceType::I, PieceType::E, PieceType::E,
                        PieceType::E, PieceType::I, PieceType::E, PieceType::E,
                        PieceType::E, PieceType::I, PieceType::E, PieceType::E,
                    ]
                }
            },
        );
        map.insert(
            PieceType::J.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 3,
                    n_cols: 3,
                    data: vec![
                      PieceType::E, PieceType::J, PieceType::E,
                      PieceType::E, PieceType::J, PieceType::E,
                      PieceType::J, PieceType::J, PieceType::E,
                    ],
                },
            }
        );
        map.insert(
            PieceType::L.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 3,
                    n_cols: 3,
                    data: vec![
                      PieceType::E, PieceType::L, PieceType::E,
                      PieceType::E, PieceType::L, PieceType::E,
                      PieceType::E, PieceType::L, PieceType::L,
                    ],
                },
            }
        );
        map.insert(
            PieceType::T.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 3,
                    n_cols: 3,
                    data: vec![
                      PieceType::E, PieceType::T, PieceType::E,
                      PieceType::T, PieceType::T, PieceType::T,
                      PieceType::E, PieceType::E, PieceType::E,
                    ],
                },
            }
        );
        map.insert(
            PieceType::O.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 2,
                    n_cols: 2,
                    data: vec![
                        PieceType::O, PieceType::O,
                        PieceType::O, PieceType::O,
                    ],
                },
            }
        );
        map.insert(
            PieceType::S.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 3,
                    n_cols: 3,
                    data: vec![
                        PieceType::E, PieceType::E, PieceType::E,
                        PieceType::E, PieceType::S, PieceType::S,
                        PieceType::S, PieceType::S, PieceType::E,
                    ],
                },
            }
        );
        map.insert(
            PieceType::Z.as_ref(),
            Piece {
                shape: Vec2D {
                    n_rows: 3,
                    n_cols: 3,
                    data: vec![
                        PieceType::E, PieceType::E, PieceType::E,
                        PieceType::Z, PieceType::Z, PieceType::E,
                        PieceType::E, PieceType::Z, PieceType::Z,
                    ],
                },
            }
        );
        map
    };
}

const KEY: &'static str = "yew.tetris.self";
const POSITION_INIT: Position = Position { x: 4, y: -1 };

pub struct Model {
    link: ComponentLink<Self>,
    storage: StorageService,
    interval: IntervalService,
    job: Option<Box<dyn Task>>,
    callback_tick: Callback<()>,
    state: State,
}

#[derive(Debug, EnumIter, AsRefStr, Clone, PartialEq, Serialize, Deserialize)]
enum PieceType {
    E,
    I,
    J,
    L,
    T,
    O,
    S,
    Z,
    TMP,
}

#[derive(Clone, Debug)]
struct Piece {
    shape: Vec2D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vec2D {
    n_rows: usize,        // number of rows
    n_cols: usize,        // number of columns (redundant, since we know the length of data)
    data: Vec<PieceType>, // data stored in a contiguous 1D array
}

impl Vec2D {
    fn set(&mut self, row: usize, col: usize, piece: &str) {
        let piece = PieceType::iter().find(|p| p.as_ref() == piece);
        if let Some(piece) = piece {
            assert!(row < self.n_rows);
            assert!(col < self.n_cols);
            self.data[row * self.n_cols + col] = piece;
        }
    }

    fn get_piece_type(&self, row: usize, col: usize) -> PieceType {
        assert!(row < self.n_rows);
        assert!(col < self.n_cols);
        self.data[row * self.n_cols + col].clone()
    }

    fn get(&self, row: usize, col: usize) -> &str {
        assert!(row < self.n_rows);
        assert!(col < self.n_cols);
        self.data[row * self.n_cols + col].as_ref()
    }
}

#[derive(Serialize, Deserialize)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Serialize, Deserialize)]
struct Player {
    piece_type: PieceType,
    piece_shape: Vec2D,
    position: Position,
    collided: bool,
}

#[derive(Serialize, Deserialize)]
struct GameStatus {
    level: usize,
    rows_cleared: usize,
    score: usize,
    game_over: bool,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    entries: Vec<Entry>,
    filter: Filter,
    value: String,
    edit_value: String,
    stage: Vec2D,
    player: Player,
    game_status: GameStatus,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    description: String,
    completed: bool,
    editing: bool,
}

pub enum Controls {
    Left,
    Right,
    Down,
    Bottom,
    Rotate,
    Pause,
}

pub enum Msg {
    Move(Controls),
    StartPause,
    StartInterval,
    Cancel,
    Tick,
}

fn initialize_stage(rows: usize, columns: usize) -> Vec2D {
    let stage: Vec2D = Vec2D {
        n_rows: rows,
        n_cols: columns,
        data: (0..rows * columns).map(|_| PieceType::E).collect(),
    };
    stage
}

fn initialize_player() -> Player {
    let random_piece: PieceType = get_random_piece();
    let piece_shape = PIECES.get(random_piece.as_ref()).unwrap().shape.clone();
    let player: Player = Player {
        piece_type: random_piece,
        piece_shape: piece_shape,
        position: POSITION_INIT,
        collided: false,
    };
    player
}

fn initialize_game_status() -> GameStatus {
    let game: GameStatus = GameStatus {
        level: 16,
        rows_cleared: 0,
        score: 0,
        game_over: false,
    };
    game
}

fn get_random_piece() -> PieceType {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(0, 7);
    info!("random number: {}", num);
    let piece: PieceType = match num {
        0 => PieceType::I,
        1 => PieceType::J,
        2 => PieceType::L,
        3 => PieceType::T,
        4 => PieceType::O,
        5 => PieceType::S,
        _ => PieceType::Z,
    };
    piece
}

pub fn fibonacci(n: usize) -> f64 {
    let n = n + 3;
    if n == 0 {
        panic!("zero is not a right argument to fibonacci()!");
    } else if n == 1 {
        return 1.0;
    }

    let mut sum = 0.0;
    let mut last = 0.0;
    let mut curr = 1.0;
    for _ in 1..n + 1 {
        sum = last + (curr / 2.0);
        last = curr;
        curr = sum;
    }

    sum
}

fn get_duration(level: usize) -> f64 {
    let mut sum: f64 = 1000.0;
    for i in 6..7 + level {
        sum = sum - (1000.0 / fibonacci(i));
    }
    info!("final sum: {}", sum);
    sum
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);

        let interval = IntervalService::new();

        let entries = {
            if let Json(Ok(restored_model)) = storage.restore(KEY) {
                restored_model
            } else {
                Vec::new()
            }
        };

        let state = State {
            entries,
            filter: Filter::All,
            value: "".into(),
            edit_value: "".into(),
            stage: initialize_stage(21, 10),
            player: initialize_player(),
            game_status: initialize_game_status(),
        };

        let mut link_clone = link.clone();
        document().add_event_listener(move |event: KeyDownEvent| {
            if event.key() == "Enter" {
                link_clone.send_message(Msg::StartPause);
            } else if event.key() == "ArrowRight" {
                info!("Right key pressed");
                link_clone.send_message_batch(vec![
                    Msg::Move(Controls::Right),
                    Msg::Cancel,
                    Msg::StartInterval,
                ]);
            } else if event.key() == "ArrowLeft" {
                info!("Left key pressed");
                link_clone.send_message_batch(vec![
                    Msg::Move(Controls::Left),
                    Msg::Cancel,
                    Msg::StartInterval,
                ]);
            } else if event.key() == "ArrowDown" {
                info!("Down key pressed");
                link_clone.send_message_batch(vec![
                    Msg::Move(Controls::Bottom),
                    Msg::Cancel,
                    Msg::StartInterval,
                ]);
            } else if event.key() == "ArrowUp" {
                // TODO when checking for colision on rotation, if bottom is not allowed, move up
                info!("Up key pressed");
                // TODO only cancel/start interval when next down will colide
                link_clone.send_message_batch(vec![
                    Msg::Move(Controls::Rotate),
                    Msg::Cancel,
                    Msg::StartInterval,
                ]);
                // TODO allow rotation when position is < 0 || > max
            }
        });

        Model {
            link: link.clone(),
            storage,
            state,
            interval,
            callback_tick: link.callback(|_| Msg::Tick),
            job: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartPause => {
                if self.job.is_none() {
                    if self.state.game_status.game_over {
                        self.state.initialize_game();
                    }
                    info!("Starting game!");
                    self.link.send_message(Msg::StartInterval);
                } else {
                    info!("Pausing game");
                    self.link.send_message(Msg::Cancel);
                }
            }
            Msg::StartInterval => {
                {
                    let duration: u64 = get_duration(self.state.game_status.level) as u64;
                    info!("Duration: {}", duration);
                    let handle = self
                        .interval
                        .spawn(Duration::from_millis(duration), self.callback_tick.clone());
                    self.job = Some(Box::new(handle));
                }
                info!("Interval started!");
            }
            Msg::Cancel => {
                if let Some(mut task) = self.job.take() {
                    task.cancel();
                }
                info!("Canceled");
                if self.job.is_none() {
                    info!("Job still exists!");
                }
            }
            Msg::Tick => {
                info!("Tick..");
                self.link.send_message(Msg::Move(Controls::Down));
            }
            Msg::Move(control) => {
                if !self.state.game_status.game_over {
                    match control {
                        Controls::Left => {
                            if self.is_move_allowed(Controls::Left, None) {
                                self.state.player.position.x = self.state.player.position.x - 1
                            }
                        }
                        Controls::Right => {
                            if self.is_move_allowed(Controls::Right, None) {
                                self.state.player.position.x = self.state.player.position.x + 1
                            }
                        }
                        Controls::Bottom => loop {
                            if self.is_move_allowed(Controls::Down, None) {
                                self.state.player.position.y = self.state.player.position.y + 1
                            } else {
                                if self.state.player.position.y <= 0 {
                                    self.state.game_over();
                                    self.link.send_message(Msg::Cancel);
                                } else {
                                    self.state.add_player_piece_stage();

                                    let rows = self.get_completed_rows();
                                    if rows.len() != 0 {
                                        self.state.update_game_state(rows.len());
                                        self.state.remove_rows(rows);
                                    }
                                }
                                break;
                            }
                        },
                        Controls::Down => {
                            if self.is_move_allowed(Controls::Down, None) {
                                self.state.player.position.y = self.state.player.position.y + 1
                            } else {
                                if self.state.player.position.y <= 0 {
                                    self.state.game_over();
                                    self.link.send_message(Msg::Cancel);
                                } else {
                                    self.state.add_player_piece_stage();

                                    let rows = self.get_completed_rows();
                                    if rows.len() != 0 {
                                        self.state.update_game_state(rows.len());
                                        self.state.remove_rows(rows);
                                    }
                                }
                            }
                        }
                        Controls::Rotate => {
                            if self.is_move_allowed(Controls::Rotate, None) {
                                self.state.rotate_player_piece();
                            } else {
                                let position = Position {
                                    x: self.state.player.position.x,
                                    y: self.state.player.position.y - 1,
                                };
                                if self.is_move_allowed(Controls::Rotate, Some(position)) {
                                    self.state.rotate_player_piece();
                                }
                            }
                        }
                        Controls::Pause => todo!(),
                    }
                }
            }
        }
        self.storage.store(KEY, Json(&self.state.entries));
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <p>{ format!("Level: {}", self.state.game_status.level) }</p>
                <p>{ format!("Rows cleared: {}", self.state.game_status.rows_cleared) }</p>
                <p>{ format!("Score: {}", self.state.game_status.score) }</p>
                <table>
                <>
            { for (0..self.state.stage.n_rows).map(|row| {
                html! {
                  <>
                    <tr>
                    {
                        for (0..self.state.stage.n_cols).map(|col| {
                            let mut cell = self.state.stage.get(row, col);
                            let Position { x, y } = self.state.player.position;
                            let Vec2D { n_rows, n_cols, data } = self.state.player.piece_shape.clone();
                            let n_rows = n_rows as isize;
                            let n_cols = n_cols as isize;
                            let rowi = row as isize;
                            let coli = col as isize;
                            if (y..y + n_rows).contains(&rowi) {
                                if (x..x + n_cols).contains(&coli) {
                                    let player_cell = self.state.player.piece_shape.get((rowi - y) as usize, (coli - x) as usize);
                                    if player_cell != PieceType::E.as_ref() {
                                        cell = player_cell;
                                    }
                                }
                            }

                            html! {
                                <td class=format!("cell-{}", cell)/>
                            }
                        })
                    }
                    </tr>
                  </>
                }
            })}
              </>
              </table>
            { if self.state.game_status.game_over {
                html! {
                    <>
                    <div class="game-over">
                    {"Game Over"}
                    </div>
                        <p>{"Press Enter to start over"}</p>
                        </>
                }
            } else {
                html! {}
            }
            }
            </div>
        }
    }
}

impl Model {
    fn is_position_empty(&self, x: isize, y: isize, player_piece: Option<Vec2D>) -> bool {
        let piece: Vec2D;
        if let Some(player_piece) = player_piece {
            piece = player_piece;
        } else {
            piece = self.state.player.piece_shape.clone();
        }
        let Vec2D {
            n_rows: stage_rows,
            n_cols: stage_cols,
            ..
        } = self.state.stage.clone();
        let Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            ..
        } = piece;
        let player_rows = player_rows as isize;
        let player_cols = player_cols as isize;
        let stage_rows = stage_rows as isize;
        let stage_cols = stage_cols as isize;

        for n_row in 0..player_rows {
            for n_col in 0..player_cols {
                let row = n_row + y;
                let col = n_col + x;

                if row < 0 || row >= stage_rows || col < 0 || col >= stage_cols {
                    info!("nope");
                } else {
                    let stage_cell = self.state.stage.get(row as usize, col as usize);
                    let player_cell = piece.get(n_row as usize, n_col as usize);
                    if stage_cell != PieceType::E.as_ref() && player_cell != PieceType::E.as_ref() {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn is_player_position_valid(&self, x: isize, y: isize, player_piece: Option<Vec2D>) -> bool {
        let piece: Vec2D;
        if let Some(player_piece) = player_piece {
            piece = player_piece;
        } else {
            piece = self.state.player.piece_shape.clone();
        }
        let Vec2D {
            n_rows: stage_rows,
            n_cols: stage_cols,
            ..
        } = self.state.stage.clone();
        let Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            ..
        } = piece;
        let player_rows = player_rows as isize;
        let player_cols = player_cols as isize;
        let stage_rows = stage_rows as isize;
        let stage_cols = stage_cols as isize;

        // check if piece ouside left border of stage
        if x < 0 {
            let distance: isize = x as isize / -1;
            for n_row in 0..player_rows {
                for n_col in 0..distance {
                    let cell = piece.get(n_row as usize, n_col as usize);
                    if cell != PieceType::E.as_ref() {
                        return false;
                    }
                }
            }
        }

        // check if piece ouside right border of stage
        if x + player_cols > stage_cols {
            let distance: isize = x + player_cols - stage_cols;
            for n_row in 0..player_rows {
                for n_col in (player_cols - distance)..player_cols {
                    let cell = piece.get(n_row as usize, n_col as usize);
                    if cell != PieceType::E.as_ref() {
                        return false;
                    }
                }
            }
        }

        // check if piece ouside low border of stage
        if y + player_rows > stage_rows {
            let distance: isize = y + player_rows - stage_rows;
            for n_row in (player_rows - distance)..player_rows {
                for n_col in 0..player_cols {
                    let cell = piece.get(n_row as usize, n_col as usize);
                    if cell != PieceType::E.as_ref() {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn get_completed_rows(&self) -> Vec<usize> {
        let mut full_rows: Vec<usize> = Vec::new();
        let Vec2D {
            n_rows: stage_rows,
            n_cols: stage_cols,
            ..
        } = self.state.stage.clone();
        let stage_rows = stage_rows as isize;
        let stage_cols = stage_cols as isize;

        for n_row in 0..stage_rows {
            let mut empty_cell_exists = false;
            for n_col in 0..stage_cols {
                if self.state.stage.get(n_row as usize, n_col as usize) == PieceType::E.as_ref() {
                    empty_cell_exists = true;
                }
            }

            if !empty_cell_exists {
                full_rows.push(n_row as usize);
            }
        }

        full_rows
    }

    fn is_rotate_allowed(&self) -> bool {
        let Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            ..
        } = self.state.player.piece_shape.clone();
        let Position { x, y } = self.state.player.position;

        let mut rotated_data: Vec<PieceType> = Vec::new();
        for n_col in 0..player_cols {
            for n_row in (0..player_rows).rev() {
                rotated_data.push(self.state.player.piece_shape.get_piece_type(n_row, n_col));
            }
        }
        let rotated_piece = Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            data: rotated_data,
        };

        self.is_position_empty(x, y, Some(rotated_piece.clone()))
            && self.is_player_position_valid(x, y, Some(rotated_piece.clone()))
    }

    fn is_move_allowed(&self, control: Controls, position: Option<Position>) -> bool {
        let x: isize;
        let y: isize;

        if let Some(position) = position {
            x = position.x;
            y = position.y;
        } else {
            x = self.state.player.position.x;
            y = self.state.player.position.y;
        }

        match control {
            Controls::Left => {
                if self.is_player_position_valid(x - 1, y, None)
                    && self.is_position_empty(x - 1, y, None)
                {
                    true
                } else {
                    false
                }
            }
            Controls::Right => {
                if self.is_player_position_valid(x + 1, y, None)
                    && self.is_position_empty(x + 1, y, None)
                {
                    true
                } else {
                    false
                }
            }
            Controls::Bottom | Controls::Down => {
                if self.is_player_position_valid(x, y + 1, None)
                    && self.is_position_empty(x, y + 1, None)
                {
                    true
                } else {
                    false
                }
            }
            Controls::Rotate => {
                if self.is_rotate_allowed() {
                    true
                } else {
                    false
                }
            }
            Controls::Pause => todo!(),
        }
    }
}

#[derive(EnumIter, ToString, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl<'a> Into<Href> for &'a Filter {
    fn into(self) -> Href {
        match *self {
            Filter::All => "#/".into(),
            Filter::Active => "#/active".into(),
            Filter::Completed => "#/completed".into(),
        }
    }
}

impl Filter {
    fn fit(&self, entry: &Entry) -> bool {
        match *self {
            Filter::All => true,
            Filter::Active => !entry.completed,
            Filter::Completed => entry.completed,
        }
    }
}

impl State {
    fn initialize_game(&mut self) {
        self.stage = initialize_stage(21, 10);
        self.game_status = initialize_game_status();
    }

    fn add_player_piece_stage(&mut self) {
        let Vec2D {
            n_rows: stage_rows,
            n_cols: stage_cols,
            ..
        } = self.stage.clone();
        let Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            ..
        } = self.player.piece_shape.clone();
        let Position { x, y } = self.player.position;
        let player_rows = player_rows as isize;
        let player_cols = player_cols as isize;
        let stage_rows = stage_rows as isize;
        let stage_cols = stage_cols as isize;

        for n_row in 0..player_rows {
            for n_col in 0..player_cols {
                let row = n_row + y;
                let col = n_col + x;

                if row < 0 || row > stage_rows || col < 0 || col > stage_cols {
                    info!("nope");
                } else {
                    let cell = self.player.piece_shape.get(n_row as usize, n_col as usize);
                    if cell != PieceType::E.as_ref() {
                        self.stage.set(row as usize, col as usize, cell);
                    }
                }
            }
        }
        let mut random_piece: PieceType;
        loop {
            random_piece = get_random_piece();
            if random_piece != self.player.piece_type {
                break;
            }
        }
        let piece_shape = PIECES.get(random_piece.as_ref()).unwrap().shape.clone();
        self.player.piece_type = random_piece;
        self.player.piece_shape = piece_shape;
        self.player.position.x = 4;
        self.player.position.y = 0;
    }

    fn update_game_state(&mut self, rows_cleared: usize) {
        if rows_cleared > 0 {
            let score: usize = match rows_cleared {
                1 => 40 * self.game_status.level,
                2 => 100 * self.game_status.level,
                3 => 300 * self.game_status.level,
                _ => 1200 * self.game_status.level,
            };
            let rows_cleared = self.game_status.rows_cleared + rows_cleared;
            let level: usize = (rows_cleared / 10) + 1;
            self.game_status = GameStatus {
                level,
                score: self.game_status.score + score,
                rows_cleared,
                game_over: self.game_status.game_over,
            }
        }
    }

    fn remove_rows(&mut self, rows: Vec<usize>) {
        let Vec2D {
            n_cols: stage_cols, ..
        } = self.stage.clone();
        let stage_cols = stage_cols as isize;

        for n_row in rows.clone() {
            let stage = self.stage.clone();
            for n_col in 0..stage_cols {
                for row in 0..n_row + 1 {
                    let piece = if row == 0 {
                        PieceType::E.as_ref()
                    } else {
                        stage.get(row - 1, n_col as usize).clone().as_ref()
                    };
                    self.stage.set(row, n_col as usize, piece);
                }
            }
        }
    }

    fn game_over(&mut self) {
        self.game_status.game_over = true;
    }

    fn rotate_player_piece(&mut self) {
        let Vec2D {
            n_rows: player_rows,
            n_cols: player_cols,
            ..
        } = self.player.piece_shape.clone();

        let mut rotated_data: Vec<PieceType> = Vec::new();
        for n_col in 0..player_cols {
            for n_row in (0..player_rows).rev() {
                rotated_data.push(self.player.piece_shape.get_piece_type(n_row, n_col));
            }
        }
        self.player.piece_shape.data = rotated_data;
    }
}
