use std::{
    fmt::{Display, Write},
    sync::Arc,
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Deserialize;

type Position = (usize, usize);

#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Empty,
    Cookie,
    Milk,
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Empty => '⬛',
            Self::Cookie => '🍪',
            Self::Milk => '🥛',
        };
        f.write_char(c)
    }
}
#[derive(Clone, Copy)]
enum DoneState {
    Cookie,
    Milk,
    Nothing,
}
impl Display for DoneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Cookie => "🍪 wins!",
            Self::Milk => "🥛 wins!",
            Self::Nothing => "No winner.",
        };
        f.write_str(s)
    }
}

pub struct Game {
    board: [[Tile; 4]; 4],
    is_done: Option<DoneState>,
    rng: StdRng,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = '⬜';
        for y in (0..4).rev() {
            f.write_char(w)?;
            for x in 0..4 {
                f.write_fmt(format_args!("{}", self.get_tile(x, y)))?
            }
            f.write_fmt(format_args!("{w}\n"))?
        }
        f.write_fmt(format_args!("{w}{w}{w}{w}{w}{w}\n"))?;

        if let Some(state) = self.is_done {
            f.write_fmt(format_args!("{state}\n"))?;
        }

        Ok(())
    }
}

impl Game {
    fn new() -> Self {
        let board = [[Tile::Empty; 4]; 4];
        let is_done = None;
        let rng = StdRng::seed_from_u64(2024);
        Self {
            board,
            is_done,
            rng,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn get_tile(&self, x: usize, y: usize) -> Tile {
        self.board[x][y]
    }

    fn test_win(&self, x: usize, y: usize) -> bool {
        let tile = self.get_tile(x, y);
        let t2b = [(x, 0), (x, 1), (x, 2), (x, 3)];
        let l2r = [(0, y), (1, y), (2, y), (3, y)];
        let l_d = [(0, 3), (1, 2), (2, 1), (3, 0)];
        let r_d = [(0, 0), (1, 1), (2, 2), (3, 3)];

        let t = |pos: &[Position]| -> bool {
            pos.contains(&(x, y))
                && pos
                    .iter()
                    .map(|(x, y)| self.get_tile(*x, *y))
                    .all(|t| t == tile)
        };

        t(&t2b) || t(&l2r) || t(&l_d) || t(&r_d)
    }

    fn is_done(&self) -> bool {
        self.is_done.is_some()
    }

    fn is_full(&self) -> bool {
        self.board
            .iter()
            .flatten()
            .find(|&&t| t == Tile::Empty)
            .is_none()
    }

    fn do_step(&mut self, column: usize, tile: Tile) -> Option<()> {
        let (y, find_tile) = self.board[column]
            .iter_mut()
            .enumerate()
            .find(|(_, t)| t == &&Tile::Empty)?;
        *find_tile = tile;

        if self.is_full() {
            self.is_done = Some(DoneState::Nothing);
        }

        if self.test_win(column, y) {
            self.is_done = Some(match tile {
                Tile::Cookie => DoneState::Cookie,
                Tile::Milk => DoneState::Milk,
                Tile::Empty => unreachable!(),
            });
        }
        Some(())
    }

    fn do_random(&mut self) {
        for y in (0..4).rev() {
            for x in 0..4 {
                let tile = if self.rng.gen::<bool>() {
                    Tile::Cookie
                } else {
                    Tile::Milk
                };
                self.board[x][y] = tile;
            }
        }

        for x in 0..4 {
            for y in 0..4 {
                if self.test_win(x, y) {
                    self.is_done = Some(DoneState::Cookie);
                    return;
                }
            }
        }
        self.is_done = Some(DoneState::Nothing)
    }
}

type StdMutex<T> = std::sync::Mutex<T>;
type GameState = Arc<StdMutex<Game>>;

#[derive(Deserialize)]
struct Payload {
    team: String,
    column: usize,
}
impl Payload {
    fn try_into(self) -> Option<(Tile, usize)> {
        let tile = match &*self.team {
            "cookie" => Tile::Cookie,
            "milk" => Tile::Milk,
            _ => return None,
        };
        if !(1..=4).contains(&self.column) {
            return None;
        }
        Some((tile, self.column - 1))
    }
}

async fn p1(State(game): State<GameState>) -> String {
    game.lock().unwrap().to_string()
}
async fn p2(State(game): State<GameState>) -> String {
    let mut game = game.lock().unwrap();
    game.reset();
    game.to_string()
}

async fn p3(State(game): State<GameState>, Path(payload): Path<Payload>) -> (StatusCode, String) {
    let (tile, colunm) = match payload.try_into() {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "".to_string()),
    };

    let mut game = game.lock().unwrap();
    if game.is_done() {
        return (StatusCode::SERVICE_UNAVAILABLE, game.to_string());
    }
    if let Some(_) = game.do_step(colunm, tile) {
        (StatusCode::OK, game.to_string())
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, game.to_string())
    }
}
async fn p4(State(game): State<GameState>) -> String {
    let mut game = game.lock().unwrap();
    game.do_random();
    game.to_string()
}

pub fn router() -> Router {
    let state = Arc::new(StdMutex::new(Game::new()));
    Router::new()
        .route("/12/board", get(p1))
        .route("/12/reset", post(p2))
        .route("/12/place/:team/:column", post(p3))
        .route("/12/random-board", get(p4))
        .with_state(state)
}
