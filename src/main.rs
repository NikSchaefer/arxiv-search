use dashmap::DashMap;
use lazy_static::lazy_static;
use rayon::prelude::*;
use rust_stemmers::{Algorithm, Stemmer};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

// Lazy static lets us define consts as calculated values
// that will compute on the first render and then cache
lazy_static! {
    // These are common words that just don't mean very much
    // We can remove them to save compute time
    static ref STOP_WORDS: HashSet<&'static str> = {
        vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in",
            "is", "it", "its", "of", "on", "that", "the", "to", "was", "were", "will", "with",
        ]
        .into_iter()
        .collect()
    };

    // Stemming words changes words into more recognizable terms
    // for example 'running' -> 'run' which makes it easier to search
    static ref STEMMER: Stemmer = Stemmer::create(Algorithm::English);
}

// the derive are methods we are allowed to use on this struct
// we comment out what we don't need (for now)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Paper {
    id: String,
    // submitter: String,
    // authors: String,
    title: String,
    // comments: String,
    // categories: String,
    #[serde(rename = "abstract")]
    paper_abstract: String,
    // update_date: String,
    #[serde(default)]
    tokenized_abstract: Vec<String>,
}

const API_HOME_URL: &str = "https://arxiv.org/pdf/";

fn remove_punctuation(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

// tokenizing helps us focus on what actually matters
// remove punctuation, lowercase, split into words, remove common words
fn tokenize(input: &str) -> Vec<String> {
    remove_punctuation(&input)
        .to_ascii_lowercase()
        .split_whitespace()
        .filter(|word| !STOP_WORDS.contains(word))
        .map(|word| STEMMER.stem(word).to_string())
        .collect()
}

fn main() {
    // Add total runtime tracker
    let total_runtime = Instant::now();

    // This leds us configure the concurrency framework to fit with our CPU
    rayon::ThreadPoolBuilder::new()
        .num_threads(11)
        .build_global()
        .unwrap();

    println!("Loading and parsing data...");

    // First, load the data from the dataset and parse into a vector
    let file = match fs::File::open("cleaned_dataset.json") {
        Ok(file) => file,
        Err(error) => panic!("Opening the file failed {error}"),
    };

    // Parse the json from the file
    let json: serde_json::Value = match serde_json::from_reader(file) {
        Ok(json) => json,
        Err(error) => panic!("Parsing the file failed: {error}"),
    };

    let mut papers: Vec<Paper> = match json.as_array() {
        Some(array) => array
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect(),
        None => panic!("No Papers Found"),
    };

    // get how many papers we are using, this is useful later
    let total_paper_count = papers.len();
    println!("{total_paper_count} Total Papers");

    // we store a word hash that tracks which papers have which words
    // a dashmap is concurrency safe without locking, allowing true parallelization
    let word_paper_map: DashMap<String, HashSet<String>> = DashMap::new();

    // iterate through each paper, adding to both hashes as we go
    // we use par_iter with rayon to compute the process in parallel
    papers.par_iter_mut().for_each(|paper| {
        // we tokenize the abstract, then hash the paper for each word inside it's abstract
        // this is later used to lookup when we have words to find relevant papers
        let tokenized = tokenize(&paper.paper_abstract);

        for word in &tokenized {
            word_paper_map
                .entry(word.to_string())
                .or_default()
                .insert(paper.id.clone());
        }
        paper.tokenized_abstract = tokenized;
    });

    // we store a hash of the paper_ids to easily look them up when needed
    // we do not include in previous iter for ease of use
    let paper_id_map: HashMap<String, Paper> = papers
        .into_iter()
        .map(|paper| (paper.id.clone(), paper.clone()))
        .collect();

    println!("Processing and indexing papers...");

    println!("Ready for queries after: {:.2?}", total_runtime.elapsed());

    // save model indexes

    // This is where we begin with our word dependent ranking
    let query: &str = "Dark Matter";
    // Track query execution time
    let query_runtime = Instant::now();

    // since we tokenize the papers, we have to tokenize the query
    let tokenized_query = tokenize(query);

    // this stores the calculated total score for each paper
    let mut scores: HashMap<String, f64> = HashMap::new();

    // iterate through each word in our search query, calculate TF-IDF for each and add to scores
    for word in tokenized_query {
        let ids = match word_paper_map.get(&word) {
            Some(ids) => ids,
            None => {
                println!("Unrecognized word");
                continue;
            }
        };

        // calculation for idf that we can pre-calculate
        let paper_with_word_count = ids.len();

        // calculate the inverse document frequency
        // this does not chance on a per word basis, so we can calculate it earlier
        let inverse_doc_frequency: f64 =
            (total_paper_count as f64 / (1 + paper_with_word_count) as f64).ln();

        // here we iterate over each paper, for each word, adding the TF-IDF score
        for paper_id in ids.iter() {
            let paper = match paper_id_map.get(paper_id) {
                Some(paper) => paper,
                None => continue,
            };

            // calculate term frequency
            let length = paper.tokenized_abstract.len();
            let word_count = paper
                .tokenized_abstract
                .iter()
                .filter(|&x| *x == *word)
                .count();

            let term_frequency: f64 = word_count as f64 / length as f64;
            let score = term_frequency * inverse_doc_frequency;

            *scores.entry(paper_id.clone()).or_default() += score;
        }
    }

    // final step is to rank the scores to get top scores
    let mut score_vec: Vec<(String, f64)> = scores.into_iter().collect();
    // sort by ranking score to get most relevant results (reverse sort) O(n log n)
    score_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nQuery Results (took {:.2?}):", query_runtime.elapsed());

    for (paper_id, score) in score_vec.iter().take(5) {
        if let Some(paper) = paper_id_map.get(paper_id) {
            let paper_link = API_HOME_URL.to_owned() + paper_id;
            println!(
                "Score: {:.4}, Title: {}, Link: {}",
                score, paper.title, paper_link
            )
        }
    }
    // the :.2 presents in 2 decimal places
    println!("\nTotal runtime: {:.2?}", total_runtime.elapsed());
}
