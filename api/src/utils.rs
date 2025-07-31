use unicode_segmentation::UnicodeSegmentation;

pub fn segment_text_into_sentences(text: &str) -> Vec<String> {
    // `split_sentence_bounds` gives you iterators over sentence boundaries.
    // We collect them into a Vec<String>.
    text.unicode_sentences()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()) // Remove empty strings
        .collect()
}

pub fn test_segmentation() {
    let raw_text = "This is sentence one. This is sentence two, with a comma. Third sentence!";
    let sentences = segment_text_into_sentences(raw_text);
    for s in sentences {
        println!("- {}", s);
    }
}

