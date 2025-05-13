#![allow(unused_unsafe)]

pub mod guesses;
pub mod answers;

use std::fmt::{self, Write};

const FIRST_LETTER: u8 = b'a';
const WORD_LENGTH: usize = 5;
const WORD_MASK: u8 = (1 << WORD_LENGTH) - 1;
const ALPHABET_LENGTH: usize = 26;

macro_rules! unsafe_index {
    ($list:expr, $index:expr) => {
        unsafe {
            $list.get_unchecked($index)
        }
    };
}

macro_rules! unsafe_index_mut {
    ($list:expr, $index:expr) => {
        unsafe {
            $list.get_unchecked_mut($index)
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Letter(pub u8);

impl Into<char> for &Letter {
    #[inline(always)]
    fn into(self) -> char {
        (FIRST_LETTER + self.0) as char
    }
}

impl From<char> for Letter {
    #[inline(always)]
    fn from(value: char) -> Self {
        Letter(value as u8 - FIRST_LETTER)
    }
}

impl fmt::Debug for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.into())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Word {
    letters: [Letter; WORD_LENGTH],
    unique_letters: u32,
}

impl Word {
    pub fn new(letters: [Letter; WORD_LENGTH]) -> Word {
        let mut unique_letters: u32 = 0;
        for letter in letters {
            unique_letters |= 1 << letter.0;
        }
        Word {
            letters,
            unique_letters,
        }
    }
}

impl Into<String> for &Word {
    fn into(self) -> String {
        let mut result = String::with_capacity(WORD_LENGTH);
        for letter in &self.letters {
            result.push(letter.into());
        };
        result
    }
}

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        let mut letters = [Letter(0); WORD_LENGTH];
        for (i, c) in value.chars().enumerate() {
            *unsafe_index_mut!(letters, i) = Letter(c.to_ascii_lowercase() as u8 - FIRST_LETTER);
        }
        Word::new(letters)
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = self.into();
        f.write_str(&string)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WordleState {
    pub incorrect_letters: u32,
    pub possible_letters: [Option<PossibleLetter>; ALPHABET_LENGTH],
    pub exact_letters: [Option<Letter>; WORD_LENGTH],
}

impl WordleState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn guess(&mut self, guess: &Word, answer: &Word) {
        // (number checked, number in answer)
        let mut letter_counts= [(0u8, 0u8); ALPHABET_LENGTH];
        for letter in answer.letters {
            unsafe_index_mut!(letter_counts, letter.0 as usize).1 += 1;
        }
        let mut to_check = guess.letters.map(|letter| Some(letter));
        for index in 0..WORD_LENGTH {
            let letter_mut = unsafe_index_mut!(to_check, index);
            let letter = letter_mut.unwrap();
            if *unsafe_index!(answer.letters, index) == letter {
                unsafe_index_mut!(letter_counts, letter.0 as usize).0 += 1;
                *unsafe_index_mut!(self.exact_letters, index) = *letter_mut;
                *letter_mut = None;
            }
        }
        
        for index in 0..WORD_LENGTH {
            if let Some(letter) = unsafe_index!(to_check, index) {
                let idx = letter.0 as usize;
                let count = unsafe_index_mut!(letter_counts, idx);
                count.0 += 1;
                if count.1 == 0 {
                    self.incorrect_letters |= 1 << letter.0;
                    continue;
                }
                let possible_letter_mut = unsafe_index_mut!(self.possible_letters, idx);
                if count.0 > count.1 {
                    if possible_letter_mut.is_none() {
                        *possible_letter_mut = Some(PossibleLetter {
                            count: (1, (WORD_LENGTH + 1) as u8),
                            positions: WORD_MASK,
                        });
                    }
                    let possible_letter = possible_letter_mut.as_mut().unwrap();
                    possible_letter.count.1 = count.1 + 1;
                    possible_letter.positions &= !(1 << index);
                    self.check_possible_letter(letter);
                    continue;
                }
                if possible_letter_mut.is_none() {
                    *possible_letter_mut = Some(PossibleLetter {
                        count: (1, (WORD_LENGTH + 1) as u8),
                        positions: WORD_MASK,
                    });
                }
                let possible_letter = possible_letter_mut.as_mut().unwrap();
                possible_letter.count.0 = count.0;
                possible_letter.positions &= !(1 << index);
                self.check_possible_letter(letter);
            }
        }
    }

    pub fn is_valid(&self, guess: &Word) -> bool {
        if self.incorrect_letters & guess.unique_letters != 0 {
            return false;
        }
        for index in 0..WORD_LENGTH {
            let letter = unsafe_index!(self.exact_letters, index);
            if let Some(letter) = letter {
                if letter != unsafe_index!(guess.letters, index) {
                    return false;
                }
            }
        }
        let mut letter_count: [u8; ALPHABET_LENGTH] = [0; ALPHABET_LENGTH];
        for index in 0..WORD_LENGTH {
            *unsafe_index_mut!(
                letter_count,
                unsafe_index!(guess.letters, index).0 as usize
            ) += 1;
        }
        for possible_index in 0..ALPHABET_LENGTH {
            let letter = match unsafe_index!(self.possible_letters, possible_index) {
                Some(value) => value,
                None => continue,
            };
            let count = *unsafe_index!(letter_count, possible_index);
            if count < letter.count.0 || count >= letter.count.1 {
                return false;
            }
            for index in 0..WORD_LENGTH {
                if letter.positions & 1 << index == 0 && unsafe_index!(guess.letters, index).0 == possible_index as u8 {
                    return false;
                }
            }
        }
        true
    }

    #[inline(always)]
    fn check_possible_letter(&mut self, letter: &Letter) {
        let index = letter.0 as usize;
        let possible_letter_ref = unsafe_index_mut!(self.possible_letters, index);
        let possible_letter = match possible_letter_ref {
            Some(value) => value,
            None => return,
        };
        let mut position: Option<usize> = None;
        // println!("{:b}", possible_letter.positions);
        for index in 0..WORD_LENGTH {
            if possible_letter.positions & 1 << index == 0 {
                continue;
            }
            if let Some(_) = position {
                return;
            }
            position = Some(index);
        }
        possible_letter.count.0 -= 1;
        if possible_letter.count.0 == 0 {
            *possible_letter_ref = None;
        }
        *unsafe_index_mut!(self.exact_letters, position.unwrap()) = Some(Letter(index as u8));
    }
}

impl Default for WordleState {
    fn default() -> Self {
        WordleState {
            incorrect_letters: 0,
            possible_letters: [const { None }; ALPHABET_LENGTH],
            exact_letters: [None; WORD_LENGTH],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PossibleLetter {
    // (min, max)
    count: (u8, u8),
    positions: u8,
}

impl PossibleLetter {
    pub fn new(min_count: u8, max_count: u8, positions: [bool; WORD_LENGTH]) -> Self {
        let mut position_bits = 0u8;
        for index in 0..WORD_LENGTH {
            if *unsafe_index!(positions, index) {
                position_bits |= 1 << index;
            }
        }
        PossibleLetter {
            count: (min_count, max_count),
            positions: position_bits,
        }
    }
}