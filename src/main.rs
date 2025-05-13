use std::{cmp::Ordering, collections::HashSet, fs::File, io::Write, time::Instant};
use indicatif::{ProgressBar, ProgressStyle};

use wordle::{answers::{ANSWERS, LENGTH as ANSWERS_LENGTH}, guesses::{GUESSES, LENGTH as GUESSES_LENGTH}, Word, WordleState};

fn get_guesses(state: &mut WordleState, guesses: &Vec<Word>, answers: &Vec<Word>) -> Vec<(u32, Word)> {
    let guesses_length = guesses.len();
    let mut answer_index: usize = 0;
    let words_eliminated = &mut vec![0u32; guesses_length][0..guesses_length];
    for answer in answers {
        answer_index += 1;
        let mut guess_index: usize = 0;
        if answer_index % 10 == 0 {
            // print!("\rTesting answer {answer_index} of {} ({:.2}%)", ANSWERS_LENGTH, answer_index as f32 / ANSWERS_LENGTH as f32 * 100f32);
            //stdout().flush().unwrap();
        }
        for guess in guesses {
            let mut state = state.clone();
            state.guess(guess, answer);
            for possible_answer in answers {
                if !state.is_valid(&possible_answer) {
                    words_eliminated[guess_index] += 1;
                }
            }
            guess_index += 1;
        }
    }
    let mut answers_set: HashSet<&Word> = HashSet::new();
    for answer in answers {
        answers_set.insert(answer);
    }
    let mut index = 0usize;
    let mut best_words: Vec<(u32, Word)> = words_eliminated.into_iter().map(|score| {
            index += 1;
            (*score, guesses[index - 1])
        }).collect();
    best_words.sort_by(|a, b| match a.0.cmp(&b.0) {
        Ordering::Equal => {
            let a_is_answer = answers_set.contains(&a.1);
            let b_is_answer = answers_set.contains(&b.1);
            a_is_answer.cmp(&b_is_answer)
        },
        value => value,
    });
    best_words.reverse();
    best_words
}

fn main() {
    let start = Instant::now();
    println!("Getting words...");
    let state = WordleState::new();
    let mut all_guesses = Vec::with_capacity(GUESSES_LENGTH);
    let mut all_answers = Vec::with_capacity(ANSWERS_LENGTH);
    for answer in ANSWERS {
        let word = Word::from(answer);
        let is_valid = state.is_valid(&word);
        if is_valid {
            all_guesses.push(word);
            all_answers.push(word);
        }
    }
    for guess in GUESSES {
        let word = Word::from(guess);
        if state.is_valid(&word) {
            all_guesses.push(word);
        }
    }
    let answers_len = all_answers.len();
    /*
    let guesses_length = guesses.len();
    let answers_length = answers.len();
    println!("{guesses_length} possible guesses, {answers_length} possible solutions found");
    let mut answer_index: usize = 0;
    let words_eliminated = &mut vec![0u32; guesses_length][0..guesses_length];
    let bar = ProgressBar::new(answers_length as u64);
    bar.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] {wide_bar} {pos:>7}/{len:7} ({percent_precise}%)  "
    ).unwrap());
    for answer in &answers {
        answer_index += 1;
        let mut guess_index: usize = 0;
        if answer_index % 10 == 0 {
            // print!("\rTesting answer {answer_index} of {} ({:.2}%)", ANSWERS_LENGTH, answer_index as f32 / ANSWERS_LENGTH as f32 * 100f32);
            //stdout().flush().unwrap();
        }
        bar.inc(1);
        for guess in &guesses {
            let mut state = state.clone();
            state.guess(guess, answer);
            for possible_answer in &answers {
                if !state.is_valid(&possible_answer) {
                    words_eliminated[guess_index] += 1;
                }
            }
            guess_index += 1;
        }
    }
    let end = Instant::now();
    println!("\nDone in {} seconds", end.duration_since(start).as_secs_f32());
    let mut answers_set: HashSet<&Word> = HashSet::new();
    for answer in &answers {
        answers_set.insert(answer);
    }
    let mut index = 0usize;
    let mut best_words: Vec<(u32, &Word)> = words_eliminated.into_iter().map(|score| {
        index += 1;
        (*score, &guesses[index - 1])
    }).collect();
    best_words.sort_by(|a, b| match a.0.cmp(&b.0) {
        Ordering::Equal => {
            let a_is_answer = answers_set.contains(a.1);
            let b_is_answer = answers_set.contains(b.1);
            a_is_answer.cmp(&b_is_answer)
        },
        value => value,
    });
    best_words.reverse();
    for (eliminated, word) in &best_words[0..min(100, best_words.len())] {
        let string: String = (*word).into();
        let eliminated = *eliminated as f32 / answers_length as f32;
        let mut marker = ' ';
        if answers_set.contains(word) {
            marker = '*';
        }
        println!("{marker} {} eliminates {eliminated}", string.to_ascii_uppercase())
    }
    */
    let bar = ProgressBar::new(all_answers.len() as u64);
    bar.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] {wide_bar} {pos:>7}/{len:7} ({percent_precise}%)  "
    ).unwrap());
    let mut file = File::create("./solutions.txt").unwrap();
    for (index, answer) in all_answers.iter().enumerate() {
        let mut guesses = all_guesses.clone();
        let mut answers = all_answers.clone();
        let mut state = WordleState::new();
        let mut first_guess = true;
        let mut solution: Vec<String> = vec![];
        loop {
            let guess = if first_guess {
                Word::from("roate")
            } else {
                get_guesses(&mut state, &guesses, &answers)[0].1
            };
            first_guess = false;
            let word: String = (&guess).into();
            solution.push(word);
            state.guess(&guess, answer);
            if &guess == answer {
                break;
            }
            let mut new_guesses: Vec<Word> = vec![];
            for guess in &guesses {
                if state.is_valid(guess) {
                    new_guesses.push(guess.clone());
                }
            }
            guesses = new_guesses;
            let mut new_answers: Vec<Word> = vec![];
            for answer in &answers {
                if state.is_valid(answer) {
                    new_answers.push(answer.clone());
                }
            }
            answers = new_answers;
        }
        write!(file, "{}", solution.join(",")).unwrap();
        if index < answers_len - 1 {
            writeln!(file).unwrap();
        }
        bar.inc(1);
    }
    let end = Instant::now();
    println!("\nDone in {} seconds", end.duration_since(start).as_secs_f32());
}
