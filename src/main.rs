use clap::Parser;
use colored::Colorize;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
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
    placements: Vec<Bitset>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Block {
    coord: Coord,
    offset: i32,
}

impl Block {
    pub fn new(coord: Coord, size: usize) -> Self {
        Self {
            coord,
            offset: Self::to_offset(coord, size),
        }
    }

    pub fn to_offset(coord: Coord, size: usize) -> i32 {
        coord.x + coord.y * size as i32 + coord.z * size as i32 * size as i32
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Orintaion {
    blocks: Vec<Coord>,
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
        let oris = orintaion.all_orintations();
        Self {
            piece_id,
            name,
            color,
            size: orintaion.blocks.len(),
            orintations: orintaion.all_orintations(),
            placements: oris.iter().flat_map(|ori| ori.placements()).collect(),
        }
    }

    pub fn char_id(&self) -> char {
        match self.piece_id {
            0..=9 => (self.piece_id as u8 + b'0') as char,
            10..=35 => (self.piece_id as u8 + b'A' - 10) as char,
            _ => panic!("Invalid piece id"),
        }
    }

    pub fn colored_id(&self) -> String {
        self.color.color(&self.char_id().to_string())
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
    pub fn new(blocks: Vec<Coord>) -> Self {
        Self { blocks: blocks }
    }

    fn normalise_first(&self) -> Self {
        Orintaion::new(
            self.blocks
                .iter()
                .map(|block| Coord {
                    x: block.x - self.blocks[0].x,
                    y: block.y - self.blocks[0].y,
                    z: block.z - self.blocks[0].z,
                })
                .collect(),
        )
    }

    pub fn placements(&self) -> Vec<Bitset> {
        let mut placements = Vec::new();
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    let mut valid = true;
                    let mut bits = Bitset::empty();
                    for block in self.blocks.iter() {
                        let coord = Coord {
                            x: block.x + x as i32,
                            y: block.y + y as i32,
                            z: block.z + z as i32,
                        };
                        if coord.x >= 0
                            && coord.x < SIZE as i32
                            && coord.y >= 0
                            && coord.y < SIZE as i32
                            && coord.z >= 0
                            && coord.z < SIZE as i32
                        {
                            let index = 16 * coord.z + 4 * coord.y + coord.x;
                            bits.set(index as usize);
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if valid {
                        placements.push(bits);
                    }
                }
            }
        }
        placements
    }

    pub fn normalise(&self) -> Self {
        // Find minimum coordinates
        let min_x = self.blocks.iter().map(|block| block.x).min().unwrap();
        let min_y = self.blocks.iter().map(|block| block.y).min().unwrap();
        let min_z = self.blocks.iter().map(|block| block.z).min().unwrap();

        Orintaion::new(
            self.blocks
                .iter()
                .map(|block| Coord {
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
        // orintations.iter().map(|o| o.normalise_first()).collect()
        orintations
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bitset {
    bits: u64,
}

impl Bitset {
    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn full() -> Self {
        Self { bits: !0 }
    }

    pub fn and(&self, other: &Bitset) -> Bitset {
        Bitset {
            bits: self.bits & other.bits,
        }
    }

    pub fn or(&self, other: &Bitset) -> Bitset {
        Bitset {
            bits: self.bits | other.bits,
        }
    }

    pub fn xor(&self, other: &Bitset) -> Bitset {
        Bitset {
            bits: self.bits ^ other.bits,
        }
    }

    pub fn not(&self) -> Bitset {
        Bitset { bits: !self.bits }
    }

    pub fn set(&mut self, index: usize) {
        self.bits |= 1 << index;
    }

    pub fn get(&self, index: usize) -> bool {
        self.bits & (1 << index) != 0
    }
}

impl From<u64> for Bitset {
    fn from(bits: u64) -> Self {
        Self { bits }
    }
}

const SIZE: usize = 4;

struct Placement {
    occupied: Bitset,
    placed: Vec<(usize, Bitset)>,
}

impl Placement {
    pub fn new() -> Self {
        Self {
            occupied: Bitset::empty(),
            placed: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Option<(usize, Bitset)> {
        match self.placed.pop() {
            Some((id, bits)) => {
                self.occupied = self.occupied.xor(&bits);
                Some((id, bits))
            }
            None => None,
        }
    }

    pub fn is_valid(&self, bits: Bitset) -> bool {
        bits.and(&self.occupied).bits == 0
    }

    pub fn place(&mut self, id: usize, bits: Bitset) {
        self.occupied = self.occupied.or(&bits);
        self.placed.push((id, bits));
    }
}

struct Puzzle {
    name: String,
    dim: Coord,
    pieces: Vec<Piece>,
}

impl Puzzle {
    fn read(filepath: &Path) -> io::Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut pieces = Vec::new();
        let mut lines = reader.lines();
        let top = lines.next().unwrap()?;
        let top: Vec<&str> = top.split(",").collect();
        let name = top[0];
        let dim = top[1];
        // let dim = top[1].parse::<usize>().unwrap();
        // println!("{} {}", name, dim);
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

                            Coord {
                                x: coords[0],
                                y: coords[1],
                                z: coords[2],
                            }
                        })
                        .collect(),
                ),
            ));
        }
        Ok(Puzzle {
            name: name.to_string(),
            dim: Coord::new(4, 4, 4),
            pieces,
        })
    }

    pub fn show(&self, placement: &Placement) {
        for y in (0..self.dim.y).rev() {
            for z in 0..self.dim.z {
                for x in 0..self.dim.x {
                    let index = z * self.dim.y * self.dim.x + y * self.dim.x + x;
                    if placement.occupied.get(index as usize) {
                        for (id, bits) in placement.placed.iter() {
                            if bits.get(index as usize) {
                                print!("{} ", self.pieces[*id].colored_id());
                                break;
                            }
                        }
                    } else {
                        print!(". ");
                    }
                }
                print!("  ");
            }
            println!();
        }
    }

    pub fn show_bit(&self, bits: &Bitset) {
        for y in (0..self.dim.y).rev() {
            for z in 0..self.dim.z {
                for x in 0..self.dim.x {
                    let index = z * self.dim.y * self.dim.x + y * self.dim.x + x;
                    if bits.get(index as usize) {
                        print!("X ");
                    } else {
                        print!(". ");
                    }
                }
                print!("  ");
            }
            println!();
        }
        println!();
    }
}

struct Solver {
    num_solutions: usize,
}

impl Solver {
    fn still_possible(&self, puzzle: &Puzzle, occ: &Bitset, remaining: &Vec<usize>) -> bool {
        for piece_id in remaining.iter() {
            let mut possible = false;
            let piece = &puzzle.pieces[*piece_id];
            for bits in piece.placements.iter() {
                if occ.and(bits).bits == 0 {
                    possible = true;
                    break;
                }
            }
            if !possible {
                return false;
            }
        }
        true
    }

    fn solve(&mut self, puzzle: &Puzzle, placement: &mut Placement, remaining: &Vec<usize>) {
        if remaining.is_empty() {
            puzzle.show(placement);
            println!("{}", self.num_solutions);
            self.num_solutions += 1;
            return;
        }

        for piece_id in remaining.iter() {
            let piece = &puzzle.pieces[*piece_id];
            let mut new_remaining = remaining.clone();
            new_remaining.retain(|&id| id != *piece_id);
            for bits in piece.placements.iter() {
                let occ = bits.or(&placement.occupied);
                if placement.is_valid(*bits) && self.still_possible(puzzle, &occ, &new_remaining) {
                    placement.place(piece.piece_id, *bits);
                    self.solve(puzzle, placement, &new_remaining);
                    placement.pop();
                }
            }
        }
    }

    fn corner_solve(
        &mut self,
        puzzle: &Puzzle,
        placement: &mut Placement,
        corners: &Vec<Bitset>,
        remaining: &Vec<usize>,
    ) {
        if corners.is_empty() {
            // println!("{} {}", corners.len(), remaining.len());
            self.solve(puzzle, placement, remaining);
            return;
        }

        let mut new_corners = corners.clone();
        let corner = new_corners.pop().unwrap();
        for piece_id in remaining.iter() {
            let piece = &puzzle.pieces[*piece_id];
            let mut new_remaining = remaining.clone();
            new_remaining.retain(|&id| id != *piece_id);
            for bits in piece.placements.iter() {
                let occ = bits.or(&placement.occupied);
                if placement.is_valid(*bits)
                    && bits.and(&corner).bits != 0
                    && self.still_possible(puzzle, &occ, &new_remaining)
                {
                    placement.place(piece.piece_id, *bits);
                    self.corner_solve(puzzle, placement, &new_corners, &new_remaining);
                    placement.pop();
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let puzzle = Puzzle::read(&args.puzzle).expect("Failed to read puzzle file");

    println!(
        "{} ({}x{}x{})",
        puzzle.name, puzzle.dim.x, puzzle.dim.y, puzzle.dim.z
    );
    for piece in puzzle.pieces.iter() {
        println!(
            "{} {} {} {} {}",
            piece.char_id(),
            piece.size,
            piece.colored_name(),
            piece.orintations.len(),
            piece.placements.len()
        );
    }

    let mut placement = Placement::new();
    placement.place(1, Bitset::from(0x0000000000000272));
    // placement.place(1, Bitset::from(0x0000000002720000));

    let mut corners = vec![
        Bitset::from(0x0000000000000001),
        Bitset::from(0x0000000000000008),
        Bitset::from(0x0000000000001000),
        Bitset::from(0x0000000000008000),
        Bitset::from(0x0001000000000000),
        Bitset::from(0x0008000000000000),
        Bitset::from(0x1000000000000000),
        Bitset::from(0x8000000000000000),
    ];

    let mut remaining = vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    let mut solver = Solver { num_solutions: 0 };

    solver.corner_solve(&puzzle, &mut placement, &mut corners, &mut remaining);

    // remaining.pop()

    for piece in puzzle.pieces.iter() {
        let mut count = 0;
        if piece.piece_id == 1 {
            continue;
        }

        for bits in piece.placements.iter() {
            if placement.is_valid(*bits) && bits.and(&Bitset::from(0x0000000000000001)).bits != 0 {
                // placement.place(piece.piece_id, *bits);
                count += 1;
                // break;
            }
        }
        println!("{} {}", piece.colored_id(), count);
    }

    puzzle.show(&placement);
    puzzle.show_bit(&placement.occupied);
}
