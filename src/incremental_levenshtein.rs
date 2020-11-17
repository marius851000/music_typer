/// a structure to do incremental levenshtein operation that does support poping a character.
struct IncrementalLevenshtein {
    source: Vec<char>,
    other: Vec<char>,
    cache: Vec<u32>,
    cache_backup: Vec<Option<Vec<u32>>>,
    goal: LevenshteinGoal,
}

#[derive(Debug, PartialEq)]
enum LevenshteinGoal {
    Distance,
    Position
}

impl IncrementalLevenshtein {
    fn new(source: &str, goal: LevenshteinGoal) -> Self {
        let source_vec: Vec<char> = source.chars().collect();
        let cache = (0..source_vec.len()+1).map(|x| x as u32).collect();

        Self {
            source: source_vec,
            cache_backup: Vec::new(),
            other: Vec::new(),
            cache,
            goal
        }
    }

    // mostly based on https://en.wikibooks.org/wiki/Algorithm_Implementation/Strings/Levenshtein_distance#Rust
    // where source is v1 and other is v2
    fn add_other_char(&mut self, source_char: char) {
        fn min3(v1: u32, v2: u32, v3: u32) -> u32 {
            v1.min(v2.min(v3))
        };
        fn delta(x: char, y: char) -> u32 {
            if x == y { 0 } else { 1 }
        };
        self.other.push(source_char);
        self.cache[0] = self.other.len() as u32;
        let mut lastdiag = (self.other.len()-1) as u32;
        for y in 1..self.source.len()+1 {
            let olddiag = self.cache[y];
            self.cache[y] = min3(
                self.cache[y] + 1, //TODO: should be one for a base algoritm
                self.cache[y-1] + match self.goal {
                    LevenshteinGoal::Distance => 1,
                    LevenshteinGoal::Position => 0
                }, //same as upper, upper is currently valid
                lastdiag + delta(self.source[y-1], self.other[self.other.len()-1])
            );
            lastdiag = olddiag;
        };

        let add_to_backup = if self.other.len() % 10 == 0 {
            Some(self.cache.clone())
        } else {
            None
        };

        self.cache_backup.push(add_to_backup);
    }

    fn add_other_str(&mut self, text: &str) {
        for chara in text.chars() {
            self.add_other_char(chara)
        }
    }

    fn pop_other_char(&mut self) {
        self.cache_backup.pop();
        self.other.pop();
        let mut chars_to_restore = Vec::new();
        let mut cache_is_restored = false;
        while let Some(backup) = self.cache_backup.pop() {
            if let Some(backed_cache) = backup {
                self.cache = backed_cache;
                self.cache_backup.push(None);
                cache_is_restored = true;
                break
            };
            chars_to_restore.push(self.other.pop().unwrap());
        };
        if !cache_is_restored {
            self.cache =  (0..self.source.len()+1).map(|x| x as u32).collect();
        };
        for chara in chars_to_restore.iter().rev() {
            self.add_other_char(*chara);
        };
    }

    fn distance(&self) -> u32 {
        if self.goal != LevenshteinGoal::Distance {
            panic!("a IncrementalLevenshtein made for {:?} was used for distance", self.goal);
        };
        self.cache[self.source.len()]
    }

    fn position(&self, precision: usize) -> usize {
        if self.goal != LevenshteinGoal::Position {
            panic!("a IncrementalLevenshtein made for {:?} was used for position", self.goal)
        };
        if self.other.len() <= precision as usize {
            return self.other.len()
        }
        let mut iterator_over_reversed_cache = self.cache.iter().enumerate().rev();
        let mut similarity_start_at = 0;
        if let Some(mut current_value) = iterator_over_reversed_cache.next().map(|(_, y)| *y) {
            let mut char_counter = precision;
            for (count, this_value) in iterator_over_reversed_cache {
                if current_value != *this_value {
                    if char_counter > 0 {
                        char_counter -= 1;
                        current_value = *this_value;
                    } else {
                        similarity_start_at = count;
                        break
                    };
                }
            }
        }
        similarity_start_at += precision + 1;
        similarity_start_at
    }
}

pub struct DistanceIncremental {
    levenshtein: IncrementalLevenshtein
}

impl DistanceIncremental {
    pub fn new(source: &str) -> Self {
        Self {
            levenshtein: IncrementalLevenshtein::new(source, LevenshteinGoal::Distance)
        }
    }

    pub fn add_other_str(&mut self, text: &str) {
        self.levenshtein.add_other_str(text)
    }

    pub fn add_other_char(&mut self, chara: char) {
        self.levenshtein.add_other_char(chara)
    }

    pub fn pop_other_char(&mut self) {
        self.levenshtein.pop_other_char()
    }

    pub fn distance(&self) -> u32 {
        self.levenshtein.distance()
    }
}

pub struct PositionIncremental {
    levenshtein: IncrementalLevenshtein
}

impl PositionIncremental {
    pub fn new(source: &str) -> Self {
        Self {
            levenshtein: IncrementalLevenshtein::new(source, LevenshteinGoal::Position)
        }
    }

    pub fn add_other_str(&mut self, text: &str) {
        self.levenshtein.add_other_str(text)
    }

    pub fn add_other_char(&mut self, chara: char) {
        self.levenshtein.add_other_char(chara)
    }

    pub fn pop_other_char(&mut self) {
        self.levenshtein.pop_other_char()
    }

    pub fn position(&self, precision: usize) -> usize {
        self.levenshtein.position(precision)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DistanceIncremental, PositionIncremental};

    #[test]
    fn test_score_incremental() {
        let mut inc = DistanceIncremental::new("hello");
        assert_eq!(inc.distance(), 5);
        inc.add_other_char('h');
        assert_eq!(inc.distance(), 4);
        for chara in &['i', 'l', 'l', 'o'] {
            inc.add_other_char(*chara);
        };
        assert_eq!(inc.distance(), 1);
        inc.add_other_char('u');
        assert_eq!(inc.distance(), 2);
        inc.pop_other_char();
        assert_eq!(inc.distance(), 1);


        let mut inc = DistanceIncremental::new("hi");
        inc.add_other_str("hi");
        assert_eq!(inc.distance(), 0);

        // test that pop work as expected
        println!("starting long pop test");
        let mut test_text = "Hello world this is a long tex";
        let mut inc = DistanceIncremental::new(test_text);
        let mut test_score_at = vec![];
        for chara in test_text.chars() {
            test_score_at.push(inc.distance());
            inc.add_other_char(chara);
        };

        for original_distance in test_score_at.iter().rev() {
            let original_distance = *original_distance;
            inc.pop_other_char();
            assert_eq!(original_distance, inc.distance());
        }
    }

    #[test]
    fn test_position_incremental() {
        let mut position = PositionIncremental::new("hello world");
        assert_eq!(position.position(1), 0);
        position.add_other_char('h');
        assert_eq!(position.position(1), 1);
        position.add_other_char('e');
        assert_eq!(position.position(1), 2);
        position.add_other_char('l');
        position.add_other_char('l');
        position.add_other_char('o');
        assert_eq!(position.position(1), 5);
        position.add_other_str(" world");
        assert_eq!(position.position(1), 11);
        position.pop_other_char();
        assert_eq!(position.position(1), 10);

        let mut position = PositionIncremental::new("hello world");
        position.add_other_str("lla wrld");
        assert_eq!(position.position(2), 11);
    }
}
