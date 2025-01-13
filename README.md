# ArXiv Search Engine 🔍

A blazingly fast search engine for arXiv papers using TF-IDF ranking and parallel processing. Built with Rust because I like living life on the edge (and really fast compile times).

## What is this?

This project creates a search engine for arXiv papers that:
- Indexes papers using TF-IDF (Term Frequency-Inverse Document Frequency)
- Processes data in parallel using Rayon
- Uses memory-efficient data structures like DashMap
- Stems words to improve search accuracy (e.g., "running" → "run")
- Saves the processed model for lightning-fast subsequent searches

Example search for "Philosophy":
```
Query Results (took 67.67ms):
Score: 2.0249, Title: Extended Version of "The Philosophy of the Trajectory Representation of Quantum Mechanics"
Score: 0.7087, Title: Kurt Goedel and His Universe
Score: 0.6704, Title: Science and Philosophy: A Love-Hate Relationship
...
```

## How it Works

### 1. Data Preprocessing
First, we clean the raw arXiv dataset using a Python script that:
- Reads the JSON dataset line by line
- Processes each paper's metadata
- Outputs a cleaned version ready for indexing

### 2. Paper Indexing
The main Rust program then:
1. Tokenizes paper abstracts by:
   - Removing punctuation
   - Converting to lowercase
   - Removing stop words (common words like "the", "and", "is")
   - Stemming words to their root form

2. Creates two main data structures:
   - A word map (word → set of paper IDs containing that word)
   - A paper map (paper ID → paper details)

3. Uses parallel processing to speed up indexing:
   - Rayon for parallel iteration
   - DashMap for concurrent hash map updates

### 3. TF-IDF Ranking
For each search query:
1. Tokenizes the search terms
2. For each term, calculates:
   - Term Frequency (TF): How often the word appears in a paper
   - Inverse Document Frequency (IDF): How unique the word is across all papers
3. Combines scores to rank papers by relevance

## Setup

### Prerequisites
- Rust (latest stable)
- Python 3.7+
- At least 16GB RAM

### Installation

1. Download the dataset:
Go to ArXiv Dataset on Kaggle

2. Clone the repository:
```bash
git clone https://github.com/NikSchaefer/arxiv-search
cd arxiv-search
```

3. Clean the dataset:
```bash
python preprocess.py
```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

The first run will take a while (about 80 minutes on an M3 MacBook Pro) as it builds the index. Subsequent runs will be much faster as they load the saved model.

## Performance Notes

- Initial indexing: ~80 minutes on M3 MacBook Pro
- Query time: ~70ms average
- Model size: Depends on dataset size, but expect several GB

## Future Improvements

- [ ] Add web interface
- [ ] Implement more advanced ranking algorithms
- [ ] Add support for boolean queries (AND, OR, NOT)
- [ ] Add citation graph analysis
- [ ] Make it even faster (because why not?)

## Contributing

Found a bug? Have an idea? Feel free to open an issue or submit a PR. I'm always looking to make things better (or at least more interesting).

## License

MIT - Do whatever you want with it, just don't blame me if your computer starts mining bitcoin instead of searching papers.
