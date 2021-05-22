use std::thread::sleep;
use std::time::Duration;
use std::ops::Range;
use std::{
    env, fs,
    io::{self, stdout, Read},
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

const MOVE_RIGHT: char = '>';
const MOVE_LEFT: char = '<';
const INCREMENT: char = '+';
const DECREMENT: char = '-';
const OUTPUT: char = '.';
const INPUT: char = ',';
const CONDITIONAL_START: char = '[';
const CONDITIONAL_END: char = ']';

fn main() {
    println!("Brainfuckery v0.1");

    if let Some(file_name) = env::args().nth(1) {
        if let Ok(mut file) = fs::OpenOptions::new()
            .read(true)
            .open(format!("./{}", file_name))
        {
            let mut code = String::new();
            &file.read_to_string(&mut code);

            let code: Vec<char> = code
                .trim()
                .chars()
                .filter(|c| {
                    matches!(
                        *c,
                        MOVE_RIGHT
                            | MOVE_LEFT
                            | INCREMENT
                            | DECREMENT
                            | INPUT
                            | OUTPUT
                            | CONDITIONAL_START
                            | CONDITIONAL_END
                    )
                })
                .collect();

            let mut memory = vec![0u8; 4294967295];
            let mut pointer = 0u32;

            let mut last_conditional_start: usize = 0;
            let mut seeking_conditional_start = false;
            let mut seeking_conditional_end = false;

            let mut step_delay_ms: u64 = 75;
            if let Some(delay) = env::args().nth(2) {
                if let Ok(delay) = delay.parse::<u64>() {
                    step_delay_ms = delay;
                } else {
                    println!("Warning, delay provided isn't valid");
                    println!("Using default value 75 milliseconds");
                    println!("Usage: brainfuckery <file.bf> <delay> <range>");
                }
            }

            let mut range: Range<u32> = 0..5;
            if let Some(input_range) = env::args().nth(3) {
                let splitted_range: Vec<&str> = input_range.split(":").collect();
                if splitted_range.len() == 2 {
                    if let Ok(range_start) = splitted_range.get(0).unwrap().parse() {
                        range.start = range_start;
                    } else {
                        println!("Warning, start range provided isn't valid, using default");
                    }
                    if let Ok(range_end) = splitted_range.get(1).unwrap().parse() {
                        range.end = range_end;
                    } else {
                        println!("Warning, end range provided isn't valid, using default");
                    }
                } else {
                    println!("Warning, range provided isn't valid");
                    println!("Using default range 0:5");
                    println!("Usage: brainfuckery <file.bf> <delay> <range>");
                }
            }

            let mut index: usize = 0;
            execute!(stdout(), Clear(ClearType::All)).unwrap();
            while index < code.len() {
                let mut output = (*code.get(index).unwrap()).to_string();
                output.push(' ');
                output.push_str(&format_mem(&memory, &pointer, range.clone()));

                execute!(
                    stdout(),
                    SavePosition,
                    MoveTo(0, 0),
                    Print(output),
                    RestorePosition
                )
                .unwrap();

                if seeking_conditional_start {
                    seeking_conditional_start = false;
                    index = last_conditional_start;
                }

                if seeking_conditional_end {
                    if *code.get(index).unwrap() == CONDITIONAL_END {
                        seeking_conditional_end = false;
                    }
                } else {
                    match *code.get(index).unwrap() {
                        MOVE_RIGHT => pointer += 1u32,
                        MOVE_LEFT => pointer -= 1u32,
                        INCREMENT => *memory.get_mut(pointer as usize).unwrap() += 1u8,
                        DECREMENT => *memory.get_mut(pointer as usize).unwrap() -= 1u8,
                        OUTPUT => execute!(
                            stdout(),
                            Print((*memory.get(pointer as usize).unwrap()) as char)
                        )
                        .unwrap(),
                        INPUT => {
                            let mut input = String::new();
                            if let Ok(_) = io::stdin().read_line(&mut input) {
                                *memory.get_mut(pointer as usize).unwrap() =
                                    input.chars().next().unwrap() as u8;
                            }
                        }
                        CONDITIONAL_START => {
                            last_conditional_start = index + 1;
                            seeking_conditional_start = false;
                            if *memory.get(pointer as usize).unwrap() == 0u8 {
                                seeking_conditional_end = true;
                            }
                        }
                        CONDITIONAL_END => {
                            seeking_conditional_end = false;
                            if *memory.get(pointer as usize).unwrap() != 0u8 {
                                seeking_conditional_start = true;
                                index = 0;
                            }
                        }
                        _ => (),
                    }
                }

                index += 1;
                sleep(Duration::from_millis(step_delay_ms));
            }
        } else {
            println!("Error, the file provided ({}) wasn't found", file_name);
        }
    } else {
        println!("Error, no argument provided");
        println!("Usage: brainfuckery <file.bf> <delay> <range>");
    }
}

fn format_mem(memory: &Vec<u8>, pointer: &u32, mem_positions: Range<u32>) -> String {
    let mut result = String::new();
    for i in mem_positions {
        if *pointer == i {
            result.push_str(">");
        } else {
            result.push_str(" ");
        }
        result.push('[');
        result.push_str(&format!("{:03}", &memory.get(i as usize).unwrap()));
        result.push(']');
    }
    return result;
}
