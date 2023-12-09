use std::fs::File;
use std::io::{self, prelude::*, stdout, BufReader, Write};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, poll, Event, KeyCode};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear};
use crossterm::{cursor, QueueableCommand};

#[derive(Default, Debug)]
struct Card {
    title: Option<String>,
    content: Vec<String>,
    number: usize,
}

fn render_tui(cards: Vec<Card>) -> Result<(), io::Error> {
    let (mut term_width, mut term_height) = terminal::size()?;
    let cards_iterator = cards.iter().enumerate();
    let mut current_card = cards_iterator.clone().nth(0).unwrap();
    terminal::enable_raw_mode()?;

    loop {
        while poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Resize(new_width, new_height) => {
                    term_width = new_width;
                    term_height = new_height;
                }
                Event::Key(event) => match event.code {
                    KeyCode::Left | KeyCode::Backspace | KeyCode::BackTab => {
                        if let Some(card) =
                            cards_iterator
                                .clone()
                                .nth(match current_card.0.checked_sub(1) {
                                    Some(number) => number,
                                    None => 0,
                                })
                        {
                            current_card = card;
                        }
                    }
                    KeyCode::Right | KeyCode::Enter | KeyCode::Tab => {
                        if let Some(card) = cards_iterator.clone().nth(current_card.0 + 1) {
                            current_card = card;
                        }
                    }
                    KeyCode::Char('q') => {
                        terminal::disable_raw_mode()?;
                        std::process::exit(0);
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        let mut stdout = stdout();

        stdout.queue(Clear(terminal::ClearType::All)).unwrap();

        let card_height: usize = current_card
            .1
            .content
            .iter()
            .map(|line| line.len())
            .sum::<usize>()
            / ((term_width as usize / 2) - 4)
            + if current_card.1.title.is_some() { 2 } else { 0 }
            + 4; // This is either next level ingenuity or absolute horror

        if current_card.1.title.is_some() {
            stdout
                .queue(cursor::MoveTo(
                    (term_width / 4) / 2,
                    term_height / 2 - card_height as u16,
                ))?
                .queue(Print(current_card.1.title.as_ref().unwrap()))?
                .queue(cursor::MoveTo(
                    (term_width / 4) / 2,
                    term_height / 2 - card_height as u16 + 1,
                ))?;
        } else {
            stdout.queue(cursor::MoveTo(
                (term_width / 4) / 2,
                term_height / 2 - card_height as u16,
            ))?;
        }

        stdout.queue(Print("-".repeat((term_width - term_width / 4) as usize)))?; // TODO: properly
                                                                                  // rescale sizes
        current_card.1.content.iter().enumerate().for_each(|line| {
            stdout
                .queue(cursor::MoveTo(
                    (term_width / 4) / 2,
                    term_height / 2 - card_height as u16
                        + if current_card.1.title.is_some() { 2 } else { 0 }
                        + line.0 as u16,
                ))
                .expect("Sorry man")
                .queue(Print(line.1))
                .expect("Sorry again");
        });
        /* stdout                  TODO: add card number at bottom of screen
        .queue(cursor::MoveTo(
            (term_width / 4) / 2,
            term_height / 2 - card_height as u16
                + if current_card.1.title.is_some() { 2 } else { 0 }
                + (current_card
                    .1
                    .content
                    .iter()
                    .map(|line| line.len())
                    .sum::<usize>() as u16
                    / ((term_width / 2) - 4))
                + 2,
        ))?
        .queue(Print(format!("C.{}", current_card.1.number)))?; */

        stdout.flush()?;
        thread::sleep(Duration::from_millis(16)); // refreshes at 60fps
    }
}

fn main() {
    let file = File::open("./test.md").unwrap();
    let file_buffers = vec![BufReader::new(&file), BufReader::new(&file)];

    /*let mut segment_vec: Vec<String> = vec![]; INFO: unsure if this will get used in future

    let mut split_buffer: Vec<Vec<String>> = vec![];

    for line in file_buffer.lines() {
        if line.as_ref().unwrap() == &String::from("---") {
            split_buffer.push(segment_vec.clone());
            segment_vec.clear();
        } else {
            segment_vec.push(line.unwrap());
        }
    }
    split_buffer.push(segment_vec.clone());

    let (card_front, card_back) = (&split_buffer[0], &split_buffer[1]);
    for f in card_front {
        for b in card_back {
            println!("F: {:0}, B: {:1}", f, b)
        }
    }*/

    let mut cards: Vec<Card> = vec![];
    file_buffers
        .into_iter()
        .enumerate()
        .for_each(|file_buffer| {
            let mut card: Card = Card::default();
            file_buffer
                .1
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| {
                    if line.contains("#") && card.title == None {
                        card.title = Some(line);
                    } else {
                        card.content.push(line);
                    }
                });
            card.number = file_buffer.0 + 1;
            cards.push(card)
        });

    println!("{}", cards.len());
    render_tui(cards);
}
