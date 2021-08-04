/// Splits a sentence (line) into a `Vec<String>`, minding quotation marks
/// for words with spaces in them.
///
/// Returns `None` if the sentence cannot be split safely (usually unbalanced quotation marks).
pub fn split_sentence<T: AsRef<str>>(sentence: T) -> Option<Vec<String>> {
    shell_words::split(sentence.as_ref().trim_end_matches('\n')).ok()
}

/// Joins a collection of words (`&str`) into one sentence string,
/// adding quotation marks for words with spaces in them.
pub fn join_sentence<I, S>(words: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    format!("{}\n", shell_words::join(words))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_split() {
        assert_eq!(
            split_sentence("AbC dEf GHi"),
            Some(vec!["AbC".into(), "dEf".into(), "GHi".into()])
        );
        assert_eq!(
            split_sentence("\"AbC dEf\" GHi"),
            Some(vec!["AbC dEf".into(), "GHi".into()])
        );
        assert_eq!(split_sentence("\"AbC dEf GHi"), None);
    }

    #[test]
    fn test_join() {
        assert_eq!(join_sentence(vec!["AbC", "dEf", "GHi"]), "AbC dEf GHi\n",);
        assert_eq!(join_sentence(vec!["AbC dEf", "GHi"]), "'AbC dEf' GHi\n",);
        assert_eq!(join_sentence(vec!["\"AbC dEf", "GHi"]), "'\"AbC dEf' GHi\n",);
    }
}
