struct Filter {
    bad_words: Vec<&'static str>,
    hell_array: Vec<&'static str>,
    accepted_name_chars: &'static str,
    a_string1335: &'static str,
    a_string1336: &'static str,
    a_string1337: &'static str,
    a_string1338: &'static str,
}

impl Filter {
    pub fn new() -> Self {
        //TODO
        Self{
        bad_words:vec![
            "kikkeli",
            "tussu",
            "tissi",
            "pimppa",
            "lutka",
            "persreikä",
            "kusipää",
            "nussi",
            "pimppi",
            "pippeli",
            "paska",
            "vitut",
            "vitun",
            "vittu",
            "saatana",
            "pillu",
            "perse",
            "perkele",
            "mulkku",
            "kulli",
            "huora",
            "helvetti",
            "helvetin",
            "kyrpä",
            "runkku",
            "runkkaa",
            "runkkari",
            "hintti",
            "fuck",
        ],

        hell_array: vec!["He'll", "he'll", "hell"],

        accepted_name_chars:
            "ABCDEFGHIJKLMNOPQRSTUVWXYZÅÄÖÜÁÉÍÓÚÑabcdefghijklmnopqrstuvwxyzåäöüáéíóúñ0123456789- ",
        a_string1335:
            "0123456789 l |¦!¡( @¤× ª°º¹²³ ©® ¥ßµ¢ àáâãåçèéêëìíîïñòóôõøùúûüýÿæ",
        a_string1336:
            "oizeasgtbp i iiiic aox aooize cr ybuc aaaäoceeeeiiiinooooouuuuyye",
        a_string1337: r##"¦!¡ []{}() ~ ª°º¹²³* `´""##,
        a_string1338: r##"||| |||||| - ''''''' '''"##, //not sure about escapes
                                                                    /*
                                                                     * if (var1.method1719().equalsIgnoreCase("fi")) {
                                                                        this.aString1342 = this.aString1335 + "bdgw";
                                                                        this.aString1343 = this.aString1336 + "ptkv";
                                                                    }
                                                                                                                                       * */
    }
    }

    fn method1567(i: char) -> i32 {
        match i {
            'a'..='z' => -1,
            'A'..='Z' => 1,
            _ => 0,
        }
    }

    fn method1566_array(s: &str) -> Vec<i32> {
        s.chars().map(Filter::method1567).collect()
    }

    fn mark_substrings(&self, s: &str) -> Vec<i32> {
        let mut occurrence_array = vec![0; s.len()];

        for substring in self.hell_array.iter() {
            let mut start_index = 0;

            while let Some(found_index) = s[start_index..].find(substring) {
                let start = found_index + start_index;
                let end = start + substring.len();
                occurrence_array[start..end].fill(1);
                start_index = end;
            }
        }

        occurrence_array
    }
    fn mark_words(
        input_string: &str,
        start_index: usize,
        mark_array: &mut [i32],
        ascii_array: &[i32],
        target_word: &str,
        mark: i32,
    ) {
        if let Some(mut current_index) = Filter::next_ascii(input_string, start_index, mark_array) {
            if current_index != start_index {
                return;
            }

            let input_length = input_string.chars().count();
            let word_length = target_word.chars().count();
            let mut t = 1;
            let mut previous_index = current_index;
            let mut tmp_char = match target_word.chars().nth(0) {
                Some(ch) => ch,
                None => return,
            };
            let mut should_early_return = true;
            let mut good_char_count = 0;
            let mut previous_char_ascii = 0;

            while let Some(current_char) = input_string.chars().nth(current_index) {
                if current_char == tmp_char
                    && t < word_length
                    && target_word.chars().nth(t).unwrap() == tmp_char
                {
                    t += 1;
                }

                if current_char != tmp_char {
                    if should_early_return {
                        return;
                    }

                    if t == word_length {
                        mark_array[start_index..previous_index].fill(mark);
                        return;
                    }

                    tmp_char = match target_word.chars().nth(t) {
                        Some(ch) => ch,
                        None => return,
                    };
                    if current_char != tmp_char {
                        return;
                    }

                    t += 1;
                }

                should_early_return = false;
                if mark == 1 {
                    if ascii_array.get(current_index) != Some(&0) {
                        return;
                    }

                    good_char_count += 1;
                    if good_char_count == 2 {
                        previous_char_ascii = ascii_array[current_index];
                    } else if good_char_count > 2
                        && ascii_array[current_index] != previous_char_ascii
                    {
                        return;
                    }
                }

                current_index += 1;
                if current_index == input_length {
                    if t != word_length {
                        return;
                    }

                    mark_array[start_index..current_index].fill(mark);
                    return;
                }

                previous_index = current_index;
                if let Some(next_index) =
                    Filter::next_ascii(input_string, current_index, mark_array)
                {
                    if mark == 1 && next_index > current_index {
                        if t != word_length {
                            return;
                        }

                        mark_array[start_index..previous_index].fill(mark);
                        return;
                    }

                    current_index = next_index;
                } else {
                    if t != word_length {
                        return;
                    }

                    mark_array[start_index..current_index].fill(mark);
                    return;
                }
            }
        }
    }

    fn mark_words2(&self, s: &str) -> Vec<i32> {
        let is_ascii = Filter::method1566_array(s);
        let mut markedwords = self.mark_substrings(s);

        self.find(&s, &mut markedwords, &is_ascii);
        markedwords
    }

    pub fn contains_badwords(&self, input: &str) -> bool {
        let marked = self.mark_words2(input);
        for i in marked {
            if i == -1 {
                return true;
            }
        }
        return false;
    }
    fn find(&self, input: &str, word_marks: &mut [i32], ascii_marks: &[i32]) -> Vec<i32> {
        for word in self.bad_words.iter() {
            for (index, _) in input.chars().enumerate() {
                Filter::mark_words(&input, index, word_marks, &ascii_marks, &word, -1);
            }
        }
        word_marks.to_vec()
    }
    /*
    fn method1566(s: &str) -> Vec<i32> {
        let mut var3 = vec![0; s.len()];
        for (var4, c) in s.chars().enumerate() {
            var3[var4] = Filter::method1567(c);
        }
        var3
    }*/

    fn next_ascii(input: &str, start_index: usize, var3: &[i32]) -> Option<usize> {
        let mut index = start_index;

        for (_, char) in input.char_indices().skip(start_index) {
            // println!("{:?}",char);
            if (char.is_ascii_lowercase() || char == 'ä' || char == 'ö') && var3[index] == 0 {
                return Some(index);
            }

            //index += char.len_utf8();
            index = index + 1;
        }

        None
    }
    fn next_ascii2(input: &str, start_index: usize, var3: &[i32]) -> Option<usize> {
        if var3[start_index] != 0 {
            return None;
        }
        input[start_index..]
            .chars()
            .position(|c| c.is_ascii_lowercase() || c == 'ä' || c == 'ö')
            .map(|index| start_index + index)
    }

    fn replace_chars(input_string: &str, target_chars: &str, replacement_chars: &str) -> String {
        input_string
            .chars()
            .map(|c| {
                if c.is_whitespace() {
                    c
                } else if target_chars.contains(c) {
                    let index = target_chars.chars().position(|x| x == c).unwrap();
                    replacement_chars.chars().nth(index).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }

    fn filter(&self, input_string: &str) -> String {
        let lowercase = input_string.to_lowercase();
        let numbersfiltered =
            Filter::replace_chars(&lowercase, self.a_string1335, self.a_string1336);

        let randomshit =
            Filter::replace_chars(&numbersfiltered, self.a_string1337, self.a_string1338);
        println!("{}", randomshit);

        randomshit.to_owned()
    }

    fn name_filter(&self, input: &str) -> String {
        let modified_string: String = input
            .chars()
            .map(|c| {
                if self.accepted_name_chars.contains(c) {
                    c
                } else {
                    '-'
                }
            })
            .collect();

        modified_string.trim_matches('-').trim().to_owned()
    }
}

mod tests {
    use crate::filter::Filter;

    #[test]
    fn contains_badwords() {
        let filter = Filter::new();
        assert_eq!(true, filter.contains_badwords("huora"));
        let testi = filter.filter("hu0rá");
        assert_eq!("huora", testi);
        assert_eq!(false, filter.contains_badwords("lol"));
    }
}
