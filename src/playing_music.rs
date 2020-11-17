use crate::{DistanceIncremental, PositionIncremental};

pub struct PlayingMusic {
    music_text: String,
    music_text_len: usize,
    typed_text: String,
    previous_character_was_space: bool,
    score_levenshtein: DistanceIncremental,
    position_levenshtein: PositionIncremental,
    map_transformed_to_source: Vec<usize>,
    precision: u32,
}

static IGNORED_CHARACTERS: [char; 8] = ['\n', '.', ',', '?', '!', ';', ':', '\r'];

impl PlayingMusic {
    pub fn new(music_text: String) -> Self {
        let mut music_text_fixed = String::new();
        let mut previous_char_was_space = false;
        let mut met_first_char = false;
        let mut map_transformed_to_source = Vec::new();
        for (position, chara) in music_text.chars().enumerate() {
            let chara = if IGNORED_CHARACTERS.contains(&chara) {
                ' '
            } else {
                chara
            };
            if chara == ' ' {
                previous_char_was_space = true;
            } else {
                if previous_char_was_space && met_first_char {
                    music_text_fixed.push(' ');
                    map_transformed_to_source.push(position);
                };
                met_first_char = true;
                music_text_fixed.push(chara);
                previous_char_was_space = false;
                map_transformed_to_source.push(position);
            }
        }
        let music_text_final = music_text_fixed.to_lowercase();
        Self {
            typed_text: String::new(),
            music_text_len: music_text_final.chars().count(),
            score_levenshtein: DistanceIncremental::new(&music_text_final),
            position_levenshtein: PositionIncremental::new(&music_text_final),
            music_text: music_text_final,
            previous_character_was_space: false,
            map_transformed_to_source,
            precision: 10,
        }
    }

    pub fn add_typed_text(&mut self, text: &str) {
        for char in text.chars() {
            self.add_typed_char(char)
        }
    }

    fn push_char_for_score(&mut self, chara: char) {
        self.typed_text.push(chara);
        self.score_levenshtein.add_other_char(chara);
        self.position_levenshtein.add_other_char(chara);
    }

    fn push_str_for_score(&mut self, text: &str) {
        self.typed_text.push_str(text);
        self.score_levenshtein.add_other_str(text);
        self.position_levenshtein.add_other_str(text);
    }

    fn pop_char_for_score(&mut self) {
        self.typed_text.pop();
        self.score_levenshtein.pop_other_char();
        self.position_levenshtein.pop_other_char();
    }

    pub fn add_typed_char(&mut self, chara: char) {
        if chara == '\u{8}' {
            if self.previous_character_was_space {
                self.previous_character_was_space = false;
            } else {
                self.pop_char_for_score();
            }
        } else {
            let chara = if IGNORED_CHARACTERS.contains(&chara) {
                ' '
            } else {
                chara
            };
            if chara == ' ' {
                self.previous_character_was_space = true;
            } else {
                if self.previous_character_was_space {
                    self.push_char_for_score(' ')
                }
                let str_to_add = chara.to_lowercase().collect::<String>();
                self.push_str_for_score(&str_to_add);
                self.previous_character_was_space = false;
            };
        }
    }

    pub fn get_typed_text(&self) -> &str {
        self.typed_text.as_str()
    }

    pub fn correctness(&self) -> f64 {
        let number_of_required_change = self.score_levenshtein.distance();
        let number_of_maximal_change = self.music_text_len as u32;
        let number_of_valid_character =
            if let Some(number) = number_of_maximal_change.checked_sub(number_of_required_change) {
                number
            } else {
                0
            };
        (number_of_valid_character as f64) / (number_of_maximal_change as f64)
    }

    pub fn get_position_in_source_text(&self) -> usize {
        let transformed_position = self.position_levenshtein.position(3);
        if let Some(position) = self.map_transformed_to_source.get(transformed_position) {
            *position
        } else {
            self.music_text.chars().count()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PlayingMusic;

    #[test]
    fn test_playing_music() {
        let mut playing_music = PlayingMusic::new("héllo, world".to_string());
        playing_music.add_typed_text("héllo");
        playing_music.add_typed_char(',');
        playing_music.add_typed_char(' ');
        playing_music.add_typed_text("world");
        assert_eq!(playing_music.get_typed_text(), "héllo world");
        assert!((playing_music.correctness() - 1.0) < 0.00000001);
    }
}
