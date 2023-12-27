use clap::Parser;
use colored::Colorize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Puzzle file
    puzzle: PathBuf,

    /// Returns solution to sudoku
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum Color {
    Red,
    Yellow,
    Blue,
    White,
}

#[derive(Clone, Debug, PartialEq)]
struct Piece {
    piece_id: usize,
    name: String,
    color: Color,
    size: usize,
    orintations: Vec<Orintaion>,
}

#[derive(Clone, Debug, PartialEq)]
struct Block {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct Orintaion {
    blocks: Vec<Block>,
}

impl Color {
    pub fn color(&self, str: &str) -> String {
        match self {
            Color::Red => str.red(),
            Color::Yellow => str.yellow(),
            Color::Blue => str.blue(),
            Color::White => str.white(),
        }
        .to_string()
    }
}

impl Piece {
    pub fn new(piece_id: usize, name: String, color: Color, orintaion: Orintaion) -> Self {
        Self {
            piece_id,
            name,
            color,
            size: orintaion.blocks.len(),
            orintations: orintaion.all_orintations(),
        }
    }

    pub fn char_id(&self) -> char {
        match self.piece_id {
            0..=9 => (self.piece_id as u8 + b'0') as char,
            10..=35 => (self.piece_id as u8 + b'A' - 10) as char,
            _ => panic!("Invalid piece id"),
        }
    }

    pub fn colored_name(&self) -> String {
        self.color.color(&self.name)
    }
}

enum Direction {
    Next,
    Clk,
    CClk,
}

impl Orintaion {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks: blocks }
    }

    fn normalise_first(&self) -> Self {
        Orintaion::new(
            self.blocks
                .iter()
                .map(|block| Block {
                    x: block.x - self.blocks[0].x,
                    y: block.y - self.blocks[0].y,
                    z: block.z - self.blocks[0].z,
                })
                .collect(),
        )
    }

    pub fn normalise(&self) -> Self {
        // Find minimum coordinates
        let min_x = self.blocks.iter().map(|block| block.x).min().unwrap();
        let min_y = self.blocks.iter().map(|block| block.y).min().unwrap();
        let min_z = self.blocks.iter().map(|block| block.z).min().unwrap();

        Orintaion::new(
            self.blocks
                .iter()
                .map(|block| Block {
                    x: block.x - min_x,
                    y: block.y - min_y,
                    z: block.z - min_z,
                })
                .collect(),
        )
    }

    fn rotate(&self, dir: Direction) -> Self {
        let mut ori = self.clone();
        for block in ori.blocks.iter_mut() {
            match dir {
                Direction::Next => {
                    let tmp = block.y;
                    block.y = block.z;
                    block.z = -tmp;
                }
                Direction::Clk => {
                    let tmp = block.x;
                    block.x = block.z;
                    block.z = -tmp;
                }
                Direction::CClk => {
                    let tmp = block.z;
                    block.z = block.x;
                    block.x = -tmp;
                }
            }
        }
        ori
    }

    fn similar(&self, other: &Self) -> bool {
        let mut count = 0;
        for block in self.blocks.iter() {
            for other_block in other.blocks.iter() {
                if block == other_block {
                    count += 1;
                    break;
                }
            }
        }
        count == self.blocks.len()
    }

    pub fn all_orintations(&self) -> Vec<Orintaion> {
        let mut orintations = Vec::new();
        let mut ori = self.normalise();
        orintations.push(ori.clone());
        let mut clk = true;
        for _dir in 0..6 {
            for _rot in 0..3 {
                ori = if clk {
                    ori.rotate(Direction::Clk)
                } else {
                    ori.rotate(Direction::CClk)
                }
                .normalise();
                if orintations.iter().all(|o| !o.similar(&ori)) {
                    orintations.push(ori.clone());
                }
            }
            ori = ori.rotate(Direction::Next).normalise();
            if orintations.iter().all(|o| !o.similar(&ori)) {
                orintations.push(ori.clone());
            }
            clk = !clk;
        }
        orintations.iter().map(|o| o.normalise_first()).collect()
    }
}

fn read_puzzle(filepath: &Path) -> Result<Vec<Piece>, std::io::Error> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let mut pieces = Vec::new();
    let mut lines = reader.lines();
    let top = lines.next().unwrap()?;
    let top: Vec<&str> = top.split(",").collect();
    let name = top[0];
    let dim = top[1];
    // let dim = top[1].parse::<usize>().unwrap();
    println!("{} {}", name, dim);
    for (piece_id, line) in lines.enumerate() {
        let line = line?;
        let line: Vec<&str> = line.split(",").collect();
        pieces.push(Piece::new(
            piece_id as usize,
            line[0].to_string(),
            match line[1] {
                "red" => Color::Red,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "white" => Color::White,
                _ => panic!("Invalid color"),
            },
            Orintaion::new(
                line[2]
                    .split('-')
                    .map(|block_str| {
                        let coords: Vec<i32> = block_str
                            .chars()
                            .filter_map(|c| c.to_digit(10))
                            .map(|num| num as i32)
                            .collect();

                        Block {
                            x: coords[0],
                            y: coords[1],
                            z: coords[2],
                        }
                    })
                    .collect(),
            ),
        ));
    }
    Ok(pieces)
}

fn main() {
    let args = Args::parse();
    let pieces = read_puzzle(&args.puzzle).expect("Failed to read puzzle file");

    for piece in pieces.iter() {
        println!(
            "{} {} {} {}",
            piece.char_id(),
            piece.size,
            piece.colored_name(),
            piece.orintations.len()
        );
    }
}
